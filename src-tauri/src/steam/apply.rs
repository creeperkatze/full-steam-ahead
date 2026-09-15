#[cfg(unix)]
use crate::steam::proton;
use crate::{
    backups,
    error::{io_context, AppError, AppResult},
    models::{
        ApplyProgressEvent, ApplyRequest, ApplyResult, ApplyStep, ImportCandidate, ShortcutEntry,
    },
    process,
    steam::{artwork, collections, detect, shortcuts, sources},
};
use std::{
    collections::HashSet,
    fs,
    thread::sleep,
    time::{Duration, Instant},
};

pub fn apply_plan_with_progress(
    on_progress: impl Fn(ApplyProgressEvent),
    mut request: ApplyRequest,
) -> AppResult<ApplyResult> {
    tracing::info!(
        candidates = request.candidates.len(),
        stop_steam = request.options.stop_steam,
        restart_steam = request.options.restart_steam,
        "Applying plan"
    );

    let (user, install_path) = detect::find_user_with_install(&request.plan.user_steam_id)?;

    let artwork_steps = request.candidates.len().max(1);
    let mut total = usize::from(request.options.stop_steam)
        + 1 // backups
        + artwork_steps
        + 1 // shortcuts
        + 1 // collections
        + usize::from(request.options.restart_steam);
    let mut current = 0usize;

    if request.options.stop_steam {
        current += 1;
        on_progress(ApplyProgressEvent {
            step: ApplyStep::StoppingSteam,
            current,
            total,
        });
        tracing::info!("Stopping Steam");
        stop_steam();
    }

    // Re-read after Steam stops: the preview may predate a rename or another import.
    let mut existing = shortcuts::read_shortcuts(&user.shortcuts_path)?;
    let skipped_candidates =
        retain_new_candidates(&mut request.candidates, &existing, &user.grid_path);
    total = total - artwork_steps + request.candidates.len().max(1);
    request.plan.changes.retain(|change| {
        !skipped_candidates.iter().any(|candidate| {
            !request
                .candidates
                .iter()
                .any(|kept| kept.id == candidate.id)
                && (change.id == format!("shortcut:{}", candidate.id)
                    || change.id.starts_with(&format!("artwork:{}:", candidate.id))
                    || change.id
                        == format!(
                            "collection:{}:{}",
                            candidate.source.collection_name(),
                            candidate.id
                        ))
        })
    });

    current += 1;
    on_progress(ApplyProgressEvent {
        step: ApplyStep::CreatingBackups,
        current,
        total,
    });
    let mut backups_created = Vec::new();
    for backup in &request.plan.backups {
        if !backup.source.exists() {
            continue;
        }
        // The plan round-trips through the frontend, so destinations are re-checked here.
        if !backups::is_valid_destination(&backup.destination) {
            return Err(AppError::Message(format!(
                "Refusing to write a backup outside the backups directory: {}",
                backup.destination.display()
            )));
        }
        if let Some(parent) = backup.destination.parent() {
            fs::create_dir_all(parent).map_err(io_context(parent))?;
        }
        fs::copy(&backup.source, &backup.destination).map_err(io_context(&backup.destination))?;
        tracing::debug!(src = %backup.source.display(), dst = %backup.destination.display(), "Backup created");
        backups_created.push(backup.destination.clone());
    }
    if let Some(backup_dir) = backups_created.first().and_then(|p| p.parent()) {
        backups::write_manifest(backup_dir, &request.plan.backups);
    }

    fs::create_dir_all(&user.grid_path).map_err(io_context(&user.grid_path))?;
    let mut skipped_change_ids = HashSet::new();

    if request.candidates.is_empty() {
        current += 1;
        on_progress(ApplyProgressEvent {
            step: ApplyStep::ApplyingArtwork { game_name: None },
            current,
            total,
        });
    } else {
        for candidate in &request.candidates {
            current += 1;
            on_progress(ApplyProgressEvent {
                step: ApplyStep::ApplyingArtwork {
                    game_name: Some(candidate.name.clone()),
                },
                current,
                total,
            });
            let candidate_skipped = artwork::apply_candidate_artwork(&user.grid_path, candidate)?;
            for skip in candidate_skipped {
                skipped_change_ids.insert(skip.change_id);
            }
        }
    }

    current += 1;
    on_progress(ApplyProgressEvent {
        step: ApplyStep::UpdatingShortcuts,
        current,
        total,
    });
    let new_candidates = request.candidates.iter().collect::<Vec<_>>();
    let mut additions = new_candidates
        .iter()
        .map(|candidate| sources::shortcut_from_candidate(candidate, &user.grid_path))
        .collect::<Vec<_>>();

    if request.options.add_self_shortcut {
        additions.push(super::self_shortcut::build(&user.grid_path)?);
    }

    #[cfg(unix)]
    {
        let proton_app_ids = new_candidates
            .iter()
            .zip(&additions)
            .filter(|(candidate, _)| candidate.needs_proton)
            .map(|(_, shortcut)| shortcut.app_id)
            .collect::<Vec<_>>();
        proton::setup_compat_tool_mapping(&install_path, &proton_app_ids)?;
    }

    if !additions.is_empty() {
        shortcuts::append_missing(&mut existing, additions);
        shortcuts::write_shortcuts(&user.shortcuts_path, &existing)?;
    }

    current += 1;
    on_progress(ApplyProgressEvent {
        step: ApplyStep::UpdatingCollections,
        current,
        total,
    });
    if request.options.create_collections && !request.candidates.is_empty() {
        collections::update_modern_collections(&user.collections_path, &request.candidates)?;
    }

    if request.options.restart_steam {
        current += 1;
        on_progress(ApplyProgressEvent {
            step: ApplyStep::RestartingSteam,
            current,
            total,
        });
        tracing::info!("Restarting Steam");
        if let Err(error) = process::restart_steam(&install_path) {
            tracing::warn!(%error, "Failed to restart Steam");
        }
    }

    let applied_changes = request
        .plan
        .changes
        .into_iter()
        .filter(|c| !skipped_change_ids.contains(&c.id))
        .collect();

    Ok(ApplyResult {
        applied_changes,
        backups_created,
    })
}

fn retain_new_candidates(
    candidates: &mut Vec<ImportCandidate>,
    existing: &[ShortcutEntry],
    grid_path: &std::path::Path,
) -> Vec<ImportCandidate> {
    let mut known = existing.to_vec();
    let mut skipped = Vec::new();
    candidates.retain(|candidate| {
        if super::matching::existing_shortcut(candidate, &known).is_some() {
            skipped.push(candidate.clone());
            return false;
        }
        known.push(sources::shortcut_from_candidate(candidate, grid_path));
        true
    });
    skipped
}

fn stop_steam() {
    if !is_steam_running() {
        return;
    }

    if let Err(error) = process::stop_steam() {
        tracing::warn!(%error, "Failed to run the Steam stop command");
        return;
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !is_steam_running() {
            tracing::info!("Steam stopped");
            return;
        }
        sleep(Duration::from_millis(200));
    }

    tracing::warn!("Steam did not close within 5s");
}

fn is_steam_running() -> bool {
    process::is_process_running(process::steam_process_name())
}

#[cfg(test)]
mod tests {
    use super::super::matching::tests::avatar_candidate;
    use super::*;
    use std::path::Path;

    #[test]
    fn apply_rechecks_stale_candidates_before_any_artwork_writes() {
        let candidate = avatar_candidate();
        assert!(candidate.existing_app_id.is_none());
        let existing = vec![ShortcutEntry {
            app_id: 123,
            app_name: "Avatar: Frontiers of Pandora".into(),
            launch_options: "uplay://launch/4740/0 -custom".into(),
            icon: "custom-avatar.ico".into(),
            ..Default::default()
        }];
        let before = shortcuts::serialize_shortcuts(&existing);
        let mut candidates = vec![candidate];
        let skipped = retain_new_candidates(&mut candidates, &existing, Path::new("grid"));
        assert_eq!(skipped.len(), 1);
        assert!(candidates.is_empty());
        assert_eq!(shortcuts::serialize_shortcuts(&existing), before);
    }

    #[test]
    fn new_imports_are_kept_and_duplicate_batch_entries_are_skipped() {
        let candidate = avatar_candidate();
        let mut candidates = vec![candidate.clone(), candidate];
        let skipped = retain_new_candidates(&mut candidates, &[], Path::new("grid"));
        assert_eq!(candidates.len(), 1);
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn stale_existing_id_does_not_hide_a_deleted_shortcut() {
        let mut candidate = avatar_candidate();
        candidate.existing_app_id = Some(123);
        let mut candidates = vec![candidate];
        assert!(retain_new_candidates(&mut candidates, &[], Path::new("grid")).is_empty());
        assert_eq!(candidates.len(), 1);
    }
}
