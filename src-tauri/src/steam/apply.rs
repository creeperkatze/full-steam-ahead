#[cfg(unix)]
use crate::steam::proton;
use crate::{
    backups,
    error::{io_context, AppResult},
    models::{ApplyProgressEvent, ApplyRequest, ApplyResult, ApplyStep},
    process,
    steam::{artwork, collections, detect, shortcuts},
};
use std::{
    collections::HashSet,
    fs,
    path::Path,
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
    let mut step = |step: ApplyStep| {
        current += 1;
        on_progress(ApplyProgressEvent {
            step,
            current,
            total,
        });
    };

    if request.options.stop_steam {
        step(ApplyStep::StoppingSteam);
        tracing::info!("Stopping Steam");
        stop_steam();
    }

    step(ApplyStep::CreatingBackups);
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut backups_created = backups::create(&request.plan.backups, &user)?;
    let backup_dir = backups_created
        .first()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf);
    // The Proton step appends its own backup.
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut backup_plans = request.plan.backups.clone();

    // Linked before artwork is written, since artwork is keyed by the shortcut's app id.
    let existing = shortcuts::read_shortcuts(&user.shortcuts_path)?;
    let mut candidates = request.candidates;
    shortcuts::link_existing(&mut candidates, &existing);

    fs::create_dir_all(&user.grid_path).map_err(io_context(&user.grid_path))?;
    let mut skipped_change_ids = HashSet::new();

    if candidates.is_empty() {
        step(ApplyStep::ApplyingArtwork { game_name: None });
    } else {
        for candidate in &candidates {
            step(ApplyStep::ApplyingArtwork {
                game_name: Some(candidate.name.clone()),
            });
            let candidate_skipped = artwork::apply_candidate_artwork(&user.grid_path, candidate)?;
            for skip in candidate_skipped {
                skipped_change_ids.insert(skip.change_id);
            }
        }
    }

    step(ApplyStep::UpdatingShortcuts);
    let mut updated = shortcuts::with_candidates(&existing, &candidates, &user.grid_path);
    if request.options.add_self_shortcut {
        shortcuts::upsert(
            &mut updated,
            super::self_shortcut::build(&user.grid_path)?,
            true,
        );
    }

    #[cfg(unix)]
    {
        let proton_app_ids = candidates
            .iter()
            .filter(|candidate| candidate.needs_proton)
            .map(crate::steam::candidate_app_id)
            .collect::<Vec<_>>();
        // Rewrites config.vdf. The manifest is written below, so the backup is recorded.
        if let Some(config_backup) = proton::setup_compat_tool_mapping(
            &install_path,
            &proton_app_ids,
            backup_dir.as_deref(),
        )? {
            backups_created.push(config_backup.destination.clone());
            backup_plans.push(config_backup);
        }
    }

    if let Some(backup_dir) = backup_dir.as_deref() {
        backups::write_manifest(backup_dir, &backup_plans);
    }

    shortcuts::write_shortcuts(&user.shortcuts_path, &updated)?;

    step(ApplyStep::UpdatingCollections);
    if request.options.create_collections {
        collections::update_modern_collections(&user.collections_path, &candidates)?;
    }

    if request.options.restart_steam {
        step(ApplyStep::RestartingSteam);
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
