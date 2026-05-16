use crate::{
    error::{io_context, AppError, AppResult},
    models::{SteamInstallation, SteamUser},
    process,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};

const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

#[derive(Debug, Deserialize)]
struct LoginUsers {
    users: std::collections::HashMap<String, LoginUser>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LoginUser {
    account_name: Option<String>,
    persona_name: Option<String>,
}

pub fn find_user(steam_id: &str) -> AppResult<SteamUser> {
    detect_steam()?
        .users
        .into_iter()
        .find(|u| u.steam_id == steam_id)
        .ok_or_else(|| AppError::UserNotFound(steam_id.to_string()))
}

pub fn detect_steam() -> AppResult<SteamInstallation> {
    let install_path = find_steam_install_path().ok_or(AppError::SteamNotFound)?;
    tracing::debug!(path = %install_path.display(), "Steam installation found");

    let userdata = install_path.join("userdata");
    let login_users = read_login_users(&install_path);
    let mut users = Vec::new();

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

            users.push(SteamUser {
                account_name: login_users.as_ref().and_then(|users| {
                    login_user_for_userdata_id(users, &steam_id)
                        .and_then(|user| user.display_name())
                }),
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

    users.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));
    tracing::debug!(users = users.len(), "Steam users discovered");

    Ok(SteamInstallation {
        install_path,
        users,
        running: is_steam_running(),
    })
}

fn find_steam_install_path() -> Option<PathBuf> {
    platform_steam_install_path().or_else(common_steam_install_path)
}

#[cfg(windows)]
fn platform_steam_install_path() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Valve\\Steam").ok()?;
    let path: String = key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(path.replace('/', "\\")))
}

#[cfg(target_os = "macos")]
fn platform_steam_install_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("Steam")
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_steam_install_path() -> Option<PathBuf> {
    dirs::home_dir().and_then(|home| {
        [
            home.join(".steam").join("steam"),
            home.join(".local").join("share").join("Steam"),
            home.join(".var")
                .join("app")
                .join("com.valvesoftware.Steam")
                .join(".steam")
                .join("steam"),
        ]
        .into_iter()
        .find(|path| path.exists())
    })
}

#[cfg(not(any(windows, unix)))]
fn platform_steam_install_path() -> Option<PathBuf> {
    None
}

fn common_steam_install_path() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|data| data.join("Steam"))
        .filter(|path| path.exists())
}

fn read_login_users(install_path: &std::path::Path) -> Option<LoginUsers> {
    let path = install_path.join("config").join("loginusers.vdf");
    let raw = fs::read_to_string(path).ok()?;
    parse_login_users_vdf(&raw)
}

fn parse_login_users_vdf(raw: &str) -> Option<LoginUsers> {
    let mut users = std::collections::HashMap::new();
    let mut current_id: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut persona_name: Option<String> = None;

    for line in raw.lines() {
        let tokens = quoted_tokens(line);
        match tokens.as_slice() {
            [id] if id.chars().all(|c| c.is_ascii_digit()) => {
                if let Some(previous) = current_id.take() {
                    users.insert(
                        previous,
                        LoginUser {
                            account_name: account_name.take(),
                            persona_name: persona_name.take(),
                        },
                    );
                }
                current_id = Some(id.clone());
            }
            [key, value] if key == "AccountName" => {
                account_name = Some(value.clone());
            }
            [key, value] if key == "PersonaName" => {
                persona_name = Some(value.clone());
            }
            _ => {}
        }
    }

    if let Some(previous) = current_id {
        users.insert(
            previous,
            LoginUser {
                account_name,
                persona_name,
            },
        );
    }

    Some(LoginUsers { users })
}

fn login_user_for_userdata_id<'a>(
    login_users: &'a LoginUsers,
    userdata_id: &str,
) -> Option<&'a LoginUser> {
    login_users.users.get(userdata_id).or_else(|| {
        let account_id = userdata_id.parse::<u64>().ok()?;
        let steam_id64 = account_id.checked_add(STEAM_ID64_BASE)?;
        login_users.users.get(&steam_id64.to_string())
    })
}

impl LoginUser {
    fn display_name(&self) -> Option<String> {
        self.persona_name
            .as_ref()
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                self.account_name
                    .as_ref()
                    .filter(|name| !name.trim().is_empty())
            })
            .cloned()
    }
}

fn quoted_tokens(line: &str) -> Vec<String> {
    line.split('"')
        .enumerate()
        .filter(|&(index, _part)| index % 2 == 1)
        .map(|(_index, part)| part.to_string())
        .collect()
}

fn is_steam_running() -> bool {
    process::is_process_running(process::steam_process_name())
}
