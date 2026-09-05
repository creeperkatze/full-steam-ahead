#[cfg(unix)]
use crate::steam::proton;
use crate::{
    backups,
    error::{io_context, AppError, AppResult},
    models::{ApplyProgressEvent, ApplyRequest, ApplyResult, ApplyStep},
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
    request: ApplyRequest,
) -> AppResult<ApplyResult> {
    tracing::info!(
        candidates = request.candidates.len(),
        stop_steam = request.options.stop_steam,
        restart_steam = request.options.restart_steam,
        "Applying plan"
    );

    let (user, install_path) = detect::find_user_with_install(&request.plan.user_steam_id)?;

    let artwork_steps = request.candidates.len().max(1);
    let total = usize::from(request.options.stop_steam)
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
    let mut existing = shortcuts::read_shortcuts(&user.shortcuts_path)?;
    let new_candidates = request
        .candidates
        .iter()
        .filter(|candidate| candidate.existing_app_id.is_none())
        .collect::<Vec<_>>();
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

    shortcuts::append_missing(&mut existing, additions);
    shortcuts::write_shortcuts(&user.shortcuts_path, &existing)?;

    current += 1;
    on_progress(ApplyProgressEvent {
        step: ApplyStep::UpdatingCollections,
        current,
        total,
    });
    if request.options.create_collections {
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
