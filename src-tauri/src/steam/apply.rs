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
        stop_steam()?;
        tracing::info!("Steam stopped");
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
            let candidate_skipped = artwork::apply_candidate_artwork(
                &user.grid_path,
                candidate,
                request.options.replace_existing_artwork,
            )?;
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
    let additions = request
        .candidates
        .iter()
        .filter(|candidate| candidate.existing_app_id.is_none())
        .map(|candidate| sources::shortcut_from_candidate(candidate, &user.grid_path))
        .collect::<Vec<_>>();
    shortcuts::append_missing(&mut existing, additions);
    shortcuts::write_shortcuts(&user.shortcuts_path, &existing)?;

    current += 1;
    on_progress(ApplyProgressEvent {
        step: ApplyStep::UpdatingCollections,
        current,
        total,
    });
    collections::update_modern_collections(&user.collections_path, &request.candidates)?;

    if request.options.restart_steam {
        current += 1;
        on_progress(ApplyProgressEvent {
            step: ApplyStep::RestartingSteam,
            current,
            total,
        });
        tracing::info!("Restarting Steam");
        let _ = process::restart_steam(&install_path);
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

fn stop_steam() -> AppResult<()> {
    if !is_steam_running() {
        return Ok(());
    }

    let output = process::stop_steam().map_err(|source| AppError::Io {
        path: process::steam_process_name().into(),
        source,
    })?;

    if !output.status.success() && is_steam_running() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let details = if stderr.is_empty() { stdout } else { stderr };
        tracing::error!(%details, "Steam stop command failed");
        return Err(AppError::Message(format!(
            "Steam could not be stopped before applying. {details}"
        )));
    }

    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if !is_steam_running() {
            return Ok(());
        }
        sleep(Duration::from_millis(300));
    }

    Err(AppError::Message(
        "Steam was asked to close, but steam.exe was still running after 15 seconds.".to_string(),
    ))
}

fn is_steam_running() -> bool {
    process::is_process_running(process::steam_process_name())
}
