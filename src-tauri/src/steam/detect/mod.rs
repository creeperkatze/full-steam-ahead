mod install;
mod users;

use crate::{
    error::{io_context, AppError, AppResult},
    models::{SteamInstallation, SteamUser},
    process,
};
use std::{fs, path::PathBuf};

pub fn find_user(steam_id: &str) -> AppResult<SteamUser> {
    detect_steam()?
        .users
        .into_iter()
        .find(|u| u.steam_id == steam_id)
        .ok_or_else(|| AppError::UserNotFound(steam_id.to_string()))
}

pub fn find_user_with_install(steam_id: &str) -> AppResult<(SteamUser, PathBuf)> {
    let install = detect_steam()?;
    let install_path = install.install_path.clone();
    let user = install
        .users
        .into_iter()
        .find(|u| u.steam_id == steam_id)
        .ok_or_else(|| AppError::UserNotFound(steam_id.to_string()))?;
    Ok((user, install_path))
}

pub fn detect_steam() -> AppResult<SteamInstallation> {
    let install_path = install::find_steam_install_path().ok_or(AppError::SteamNotFound)?;
    tracing::debug!(path = %install_path.display(), "Steam installation found");

    let userdata = install_path.join("userdata");
    let login_users = users::read_login_users(&install_path);
    let mut steam_users = Vec::new();

    if userdata.exists() {
        for entry in fs::read_dir(&userdata).map_err(io_context(&userdata))? {
            let entry = entry.map_err(io_context(&userdata))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let Some(steam_id) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };

            let login_user = login_users
                .as_ref()
                .and_then(|lu| users::login_user_for_userdata_id(lu, &steam_id));

            let avatar_path = steam_id
                .parse::<u64>()
                .ok()
                .and_then(|id| id.checked_add(users::STEAM_ID64_BASE))
                .map(|id64| {
                    install_path
                        .join("config")
                        .join("avatarcache")
                        .join(format!("{id64}.png"))
                })
                .filter(|p| p.exists());

            steam_users.push(SteamUser {
                account_name: login_user.and_then(|u| u.display_name()),
                avatar_path,
                shortcuts_path: path.join("config").join("shortcuts.vdf"),
                grid_path: path.join("config").join("grid"),
                collections_path: path
                    .join("config")
                    .join("cloudstorage")
                    .join("cloud-storage-namespace-1.json"),
                steam_id,
            });
        }
    }

    steam_users.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));
    tracing::debug!(users = steam_users.len(), "Steam users discovered");

    Ok(SteamInstallation {
        install_path,
        users: steam_users,
        running: is_steam_running(),
    })
}

fn is_steam_running() -> bool {
    process::is_process_running(process::steam_process_name())
}
