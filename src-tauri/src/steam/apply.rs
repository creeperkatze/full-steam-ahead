use crate::{
    error::{io_context, AppError, AppResult},
    models::{ApplyRequest, ApplyResult},
    steam::{artwork, collections, detect, shortcuts, sources},
};
use std::{fs, process::Command};

pub fn apply_plan(request: ApplyRequest) -> AppResult<ApplyResult> {
    let install = detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == request.plan.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.plan.user_steam_id.clone()))?;

    if request.options.stop_steam {
        let _ = Command::new("taskkill").args(["/IM", "steam.exe"]).output();
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
