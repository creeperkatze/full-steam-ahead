use crate::{
    error::{io_context, AppError, AppResult},
    models::{ApplyProgressEvent, ApplyRequest, ApplyResult},
    process,
    steam::{artwork, collections, detect, shortcuts, sources},
};
use std::{
    fs,
    thread::sleep,
    time::{Duration, Instant},
};
use tauri::Emitter;

pub fn apply_plan_with_progress(
    app: &tauri::AppHandle,
    request: ApplyRequest,
) -> AppResult<ApplyResult> {
    let install = detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == request.plan.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.plan.user_steam_id.clone()))?;

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
        let _ = app.emit(
            "apply-progress",
            ApplyProgressEvent {
                step: "Stopping Steam".into(),
                current,
                total,
            },
        );
        stop_steam()?;
    }

    current += 1;
    let _ = app.emit(
        "apply-progress",
        ApplyProgressEvent {
            step: "Creating backups".into(),
            current,
            total,
        },
    );
    let mut backups_created = Vec::new();
    for backup in &request.plan.backups {
        if !backup.source.exists() {
            continue;
        }
        if let Some(parent) = backup.destination.parent() {
            fs::create_dir_all(parent).map_err(io_context(parent))?;
        }
        fs::copy(&backup.source, &backup.destination).map_err(io_context(&backup.destination))?;
        backups_created.push(backup.destination.clone());
    }

    fs::create_dir_all(&user.grid_path).map_err(io_context(&user.grid_path))?;
    let mut skipped_changes = Vec::new();

    if request.candidates.is_empty() {
        current += 1;
        let _ = app.emit(
            "apply-progress",
            ApplyProgressEvent {
                step: "Applying artwork".into(),
                current,
                total,
            },
        );
    } else {
        for candidate in &request.candidates {
            current += 1;
            let _ = app.emit(
                "apply-progress",
                ApplyProgressEvent {
                    step: format!("Downloading artwork for {}", candidate.name),
                    current,
                    total,
                },
            );
            let candidate_skipped = artwork::apply_candidate_artwork(
                &user.grid_path,
                candidate,
                request.options.replace_existing_artwork,
            )?;
            skipped_changes.extend(candidate_skipped);
        }
    }

    current += 1;
    let _ = app.emit(
        "apply-progress",
        ApplyProgressEvent {
            step: "Updating shortcuts".into(),
            current,
            total,
        },
    );
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
    let _ = app.emit(
        "apply-progress",
        ApplyProgressEvent {
            step: "Updating collections".into(),
            current,
            total,
        },
    );
    collections::update_modern_collections(&user.collections_path, &request.candidates)?;

    if request.options.restart_steam {
        current += 1;
        let _ = app.emit(
            "apply-progress",
            ApplyProgressEvent {
                step: "Restarting Steam".into(),
                current,
                total,
            },
        );
        let _ = process::restart_steam(&install.install_path);
    }

    Ok(ApplyResult {
        applied_changes: request.plan.changes,
        backups_created,
        skipped_changes,
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
