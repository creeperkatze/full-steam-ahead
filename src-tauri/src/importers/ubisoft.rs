use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    #[cfg(windows)]
    return scan_windows(user);

    #[cfg(unix)]
    return scan_unix(user);

    #[cfg(not(any(windows, unix)))]
    Ok(Vec::new())
}

#[cfg(windows)]
fn scan_windows(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(launcher_key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher") else {
        return Ok(Vec::new());
    };
    let launcher_path = launcher_key
        .get_value::<String, _>("InstallDir")
        .ok()
        .and_then(|dir| launcher_from_dir(Path::new(&dir)))
        .or_else(default_launcher_path);
    let Some(launcher_path) = launcher_path else {
        return Ok(Vec::new());
    };

    let Ok(installs) =
        hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs")
    else {
        return Ok(Vec::new());
    };

    let mut candidates = Vec::new();
    for id in installs.enum_keys().flatten() {
        let Ok(install) = installs.open_subkey(&id) else {
            continue;
        };
        let Ok(install_dir): Result<String, _> = install.get_value("InstallDir") else {
            continue;
        };
        if !Path::new(&install_dir).exists() {
            continue;
        }

        let uninstall_path = format!(
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Uplay Install {id}"
        );
        let title = hklm
            .open_subkey(uninstall_path)
            .ok()
            .and_then(|k| k.get_value::<String, _>("DisplayName").ok())
            .unwrap_or_else(|| format!("Ubisoft game {id}"));

        candidates.push(launcher_candidate(
            user,
            ImportSource::UbisoftConnect,
            "ubisoft",
            title,
            launcher_path.clone(),
            format!("uplay://launch/{id}/0"),
            vec!["Ubisoft Connect".to_string()],
        ));
    }

    Ok(candidates)
}

#[cfg(windows)]
fn launcher_from_dir(dir: &Path) -> Option<PathBuf> {
    ["UbisoftConnect.exe", "upc.exe"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.exists())
}

#[cfg(windows)]
fn default_launcher_path() -> Option<PathBuf> {
    let pf_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
    launcher_from_dir(
        &Path::new(&pf_x86)
            .join("Ubisoft")
            .join("Ubisoft Game Launcher"),
    )
}

#[cfg(unix)]
fn scan_unix(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(launcher_info) = find_uplay_launcher_unix() else {
        return Ok(Vec::new());
    };

    let config_file = launcher_info
        .exe_path
        .parent()
        .unwrap_or(Path::new("/"))
        .join("cache")
        .join("configuration")
        .join("configurations");

    let Ok(buffer) = std::fs::read(&config_file) else {
        return Ok(Vec::new());
    };

    let data_dir = launcher_info
        .exe_path
        .parent()
        .unwrap_or(Path::new("/"))
        .join("data")
        .join("games");

    let splits = split_config_sections(&buffer);
    let candidates = splits
        .iter()
        .filter(|s| is_valid_game_config(s))
        .flat_map(|s| parse_game_configs(s))
        .map(|game| {
            let icon = data_dir.join(game.icon_image).to_string_lossy().to_string();
            let id = game
                .register
                .strip_prefix(
                    "HKEY_LOCAL_MACHINE\\SOFTWARE\\Ubisoft\\Launcher\\Installs\\",
                )
                .unwrap_or_default()
                .strip_suffix("\\InstallDir")
                .unwrap_or_default()
                .to_string();

            // On Unix with Proton, embed compat path into launch options
            let launch_url = if let Some(ref compat) = launcher_info.compat_folder {
                format!(
                    "STEAM_COMPAT_DATA_PATH=\"{}\" %command% -'uplay://launch/{id}/0'",
                    compat.display()
                )
            } else {
                format!("uplay://launch/{id}/0")
            };

            let _ = icon;
            launcher_candidate(
                user,
                ImportSource::UbisoftConnect,
                "ubisoft",
                game.shortcut_name.to_string(),
                launcher_info.exe_path.clone(),
                launch_url,
                vec!["Ubisoft Connect".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

#[cfg(unix)]
struct UplayLauncherInfo {
    exe_path: PathBuf,
    compat_folder: Option<PathBuf>,
}

#[cfg(unix)]
fn find_uplay_launcher_unix() -> Option<UplayLauncherInfo> {
    let home = std::env::var("HOME").ok()?;
    let compat_dir = PathBuf::from(&home)
        .join(".steam")
        .join("steam")
        .join("steamapps")
        .join("compatdata");

    for entry in std::fs::read_dir(compat_dir).ok()?.flatten() {
        let exe = entry
            .path()
            .join("pfx")
            .join("drive_c")
            .join("Program Files (x86)")
            .join("Ubisoft")
            .join("Ubisoft Game Launcher")
            .join("upc.exe");

        let games = entry
            .path()
            .join("pfx")
            .join("drive_c")
            .join("Program Files (x86)")
            .join("Ubisoft")
            .join("Ubisoft Game Launcher")
            .join("games");

        if exe.exists() && games.exists() {
            return Some(UplayLauncherInfo {
                exe_path: exe,
                compat_folder: Some(entry.path()),
            });
        }
    }
    None
}

#[cfg(unix)]
struct GameConfig<'a> {
    icon_image: &'a str,
    shortcut_name: &'a str,
    register: &'a str,
}

#[cfg(unix)]
fn split_config_sections(buffer: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(buffer)
        .split("version: 2.0")
        .map(|s| s.replace('?', ""))
        .collect()
}

#[cfg(unix)]
fn is_valid_game_config(section: &str) -> bool {
    ["executables:", "online:", "shortcut_name:", "register:"]
        .iter()
        .all(|req| section.contains(req))
}

#[cfg(unix)]
fn parse_game_configs(section: &str) -> Vec<GameConfig<'_>> {
    let mut results = Vec::new();
    let mut icon_image = "";
    let mut shortcut_name = "";
    let mut register = "";
    let mut in_online = false;

    for line in section.lines().map(|l| l.trim()) {
        if line.starts_with("online:") {
            in_online = true;
            continue;
        }
        if line.starts_with("offline:") {
            break;
        }
        if let Some(val) = line.strip_prefix("icon_image: ") {
            if val.is_empty() {
                break;
            }
            icon_image = val;
        }
        if !in_online {
            continue;
        }
        if let Some(val) = line.strip_prefix("- shortcut_name: ") {
            if val.is_empty() {
                break;
            }
            shortcut_name = val;
            continue;
        }
        if let Some(val) = line.strip_prefix("register: ") {
            if val.is_empty() {
                break;
            }
            register = val;
            continue;
        }
        if line == "denuvo: yes" {
            results.push(GameConfig { icon_image, shortcut_name, register });
        }
    }
    results
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::*;

    #[cfg(unix)]
    #[test]
    fn split_on_version_marker() {
        let buffer = b"header?version: 2.0mid?version: 2.0end";
        let sections = split_config_sections(buffer);
        // '?' chars are stripped, splits on the marker
        assert_eq!(sections, vec!["header", "mid", "end"]);
    }

    #[cfg(unix)]
    #[test]
    fn valid_game_config_requires_all_keys() {
        let valid = "executables:\nonline:\nshortcut_name: game\nregister: HKEY...";
        assert!(is_valid_game_config(valid));

        let missing_online = "executables:\nshortcut_name: game\nregister: HKEY...";
        assert!(!is_valid_game_config(missing_online));

        let missing_register = "executables:\nonline:\nshortcut_name: game";
        assert!(!is_valid_game_config(missing_register));
    }

    #[cfg(unix)]
    #[test]
    fn parse_game_config_extracts_fields() {
        let section = "
            icon_image: game.ico
            online:
            - shortcut_name: My Game
              register: HKEY_LOCAL_MACHINE\\SOFTWARE\\Ubisoft\\Launcher\\Installs\\123\\InstallDir
              denuvo: yes
            offline:
        ";
        let games = parse_game_configs(section);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].icon_image, "game.ico");
        assert_eq!(games[0].shortcut_name, "My Game");
        assert_eq!(
            games[0].register,
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Ubisoft\\Launcher\\Installs\\123\\InstallDir"
        );
    }

    #[cfg(unix)]
    #[test]
    fn parse_game_config_stops_at_offline() {
        let section = "
            online:
            - shortcut_name: Game A
              register: HKEY...\\1\\InstallDir
              denuvo: yes
            offline:
            - shortcut_name: Should Not Appear
              register: HKEY...\\2\\InstallDir
              denuvo: yes
        ";
        let games = parse_game_configs(section);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].shortcut_name, "Game A");
    }
}
