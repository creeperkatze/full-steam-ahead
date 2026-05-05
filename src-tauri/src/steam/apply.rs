use crate::{
    error::{io_context, AppError, AppResult},
    models::{ApplyRequest, ApplyResult},
    steam::{artwork, collections, detect, shortcuts, sources},
};
use std::{
    fs,
    process::Command,
    thread::sleep,
    time::{Duration, Instant},
};

pub fn apply_plan(request: ApplyRequest) -> AppResult<ApplyResult> {
    let install = detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == request.plan.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.plan.user_steam_id.clone()))?;

    if request.options.stop_steam {
        stop_steam()?;
    }

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

    let skipped_changes = artwork::apply_artwork(
        &user.grid_path,
        &request.candidates,
        request.options.replace_existing_artwork,
    )?;

    let mut existing = shortcuts::read_shortcuts(&user.shortcuts_path)?;
    let additions = request
        .candidates
        .iter()
        .filter(|candidate| candidate.existing_app_id.is_none())
        .map(|candidate| sources::shortcut_from_candidate(candidate, &user.grid_path))
        .collect::<Vec<_>>();
    shortcuts::append_missing(&mut existing, additions);
    shortcuts::write_shortcuts(&user.shortcuts_path, &existing)?;

    if request.options.write_collections {
        collections::update_modern_collections(&user.collections_path, &request.candidates)?;
    }

    if request.options.restart_steam {
        let steam_exe = install.install_path.join("steam.exe");
        if steam_exe.exists() {
            let _ = Command::new(steam_exe).spawn();
        }
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

    let output = Command::new("taskkill")
        .args(["/F", "/T", "/IM", "steam.exe"])
        .output()
        .map_err(|source| AppError::Io {
            path: "taskkill".into(),
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
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("steam.exe"))
        .unwrap_or(false)
}
