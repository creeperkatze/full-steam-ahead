mod install;
mod users;

use crate::{
    error::{io_context, AppError, AppResult},
    models::{SteamInstallation, SteamUser},
    process,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

pub fn find_install_path() -> Option<PathBuf> {
    install::find_steam_install_path(steam_location_override().as_deref())
}

pub fn is_valid_steam_location(path: &Path) -> bool {
    install::is_steam_install(path)
}

/// Whether the detected Steam runs inside its own Flatpak sandbox.
#[cfg(unix)]
pub fn is_sandboxed_steam() -> bool {
    find_install_path()
        .as_deref()
        .is_some_and(install::is_flatpak_steam)
}

#[cfg(unix)]
const STEAM_FLATPAK_APP_ID: &str = "com.valvesoftware.Steam";
#[cfg(unix)]
const FLATPAK_HOST_TALK_NAME: &str = "org.freedesktop.Flatpak";

/// Whether sandboxed Steam is missing the permission `flatpak-spawn --host` needs.
#[cfg(unix)]
pub fn needs_flatpak_permission() -> bool {
    is_sandboxed_steam() && !steam_flatpak_permission_granted()
}

#[cfg(not(unix))]
pub fn needs_flatpak_permission() -> bool {
    false
}

/// Checks Steam's effective Flatpak permissions for the required talk-name.
#[cfg(unix)]
fn steam_flatpak_permission_granted() -> bool {
    let flatpak = crate::importers::host_binary_path("flatpak");
    let Some(output) = crate::importers::command_stdout(
        crate::importers::host_command(&flatpak.display().to_string()).args([
            "info",
            "--show-permissions",
            STEAM_FLATPAK_APP_ID,
        ]),
    ) else {
        return false;
    };

    output.contains(&format!("{FLATPAK_HOST_TALK_NAME}=talk"))
}

/// Grants sandboxed Steam permission to reach the host via `flatpak-spawn` (not on by default).
#[cfg(unix)]
pub fn grant_steam_flatpak_permission() -> AppResult<()> {
    let flatpak = crate::importers::host_binary_path("flatpak");
    let status = crate::importers::host_command(&flatpak.display().to_string())
        .args([
            "override",
            "--user",
            &format!("--talk-name={FLATPAK_HOST_TALK_NAME}"),
            STEAM_FLATPAK_APP_ID,
        ])
        .status()
        .map_err(|source| AppError::Message(format!("Failed to run flatpak override: {source}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::Message(
            "flatpak override did not complete successfully".to_string(),
        ))
    }
}

fn steam_location_override() -> Option<PathBuf> {
    crate::commands::load_settings()
        .ok()
        .and_then(|settings| settings.steam_location)
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
}

pub fn detect_steam() -> AppResult<SteamInstallation> {
    let install_path = find_install_path().ok_or_else(|| {
        tracing::warn!(
            override_path = ?steam_location_override(),
            "No Steam installation found"
        );
        AppError::SteamNotFound
    })?;

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
                .and_then(|id64| {
                    let cache = install_path.join("config").join("avatarcache");
                    ["png", "jpg", "jpeg", "gif"]
                        .iter()
                        .map(|ext| cache.join(format!("{id64}.{ext}")))
                        .find(|p| p.exists())
                });

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

    Ok(SteamInstallation {
        install_path,
        users: steam_users,
        running: is_steam_running(),
        needs_flatpak_permission: needs_flatpak_permission(),
    })
}

fn is_steam_running() -> bool {
    process::is_process_running(process::steam_process_name())
}
