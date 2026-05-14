use crate::{
    error::{io_context, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct OriginPaths {
    exe_path: PathBuf,
    local_content_path: PathBuf,
    #[cfg_attr(not(unix), allow(dead_code))]
    compat_folder: Option<PathBuf>,
}

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(paths) = find_origin_paths() else {
        return Ok(Vec::new());
    };

    let local_content = paths.local_content_path.join("LocalContent");
    if !local_content.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&local_content)
        .map_err(io_context(&local_content))?
        .flatten()
    {
        let game_folder = entry.path();
        if !game_folder.is_dir() {
            continue;
        }
        let Some(id) = read_offer_id(&game_folder) else {
            continue;
        };
        let title = game_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("EA game")
            .to_string();

        // On Unix, embed the compat path into the launch URL so Steam uses the right Proton prefix
        #[cfg(unix)]
        let launch_url = if let Some(ref compat) = paths.compat_folder {
            format!(
                "STEAM_COMPAT_DATA_PATH=\"{}\" %command% -'origin2://game/launch?offerIds={id}&autoDownload=1&authCode=&cmdParams='",
                compat.display()
            )
        } else {
            format!("origin2://game/launch?offerIds={id}&autoDownload=1&authCode=&cmdParams=")
        };

        #[cfg(not(unix))]
        let launch_url =
            format!("origin2://game/launch?offerIds={id}&autoDownload=1&authCode=&cmdParams=");

        candidates.push(launcher_candidate(
            user,
            ImportSource::Origin,
            "origin",
            title,
            paths.exe_path.clone(),
            launch_url,
            vec!["EA app / Origin".to_string()],
        ));
    }

    Ok(candidates)
}

fn find_origin_paths() -> Option<OriginPaths> {
    #[cfg(windows)]
    {
        let program_data =
            std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        let local_content = PathBuf::from(&program_data).join("Origin");
        if !local_content.exists() {
            return None;
        }
        let exe_path = origin_launcher_path()?;
        Some(OriginPaths {
            exe_path,
            local_content_path: local_content,
            compat_folder: None,
        })
    }

    #[cfg(unix)]
    {
        let home = std::env::var("HOME").ok()?;
        let compat_dir = PathBuf::from(&home)
            .join(".steam")
            .join("steam")
            .join("steamapps")
            .join("compatdata");

        for entry in std::fs::read_dir(compat_dir).ok()?.flatten() {
            let drive_c = entry.path().join("pfx").join("drive_c");

            let exe_path = drive_c
                .join("Program Files (x86)")
                .join("Origin")
                .join("Origin.exe");
            let local_content = drive_c.join("ProgramData").join("Origin");

            if exe_path.exists() && local_content.exists() {
                return Some(OriginPaths {
                    exe_path,
                    local_content_path: local_content,
                    compat_folder: Some(entry.path()),
                });
            }
        }
        None
    }
}

#[cfg(windows)]
fn origin_launcher_path() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};
    let command: String = RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey("eadm\\shell\\open\\command")
        .ok()?
        .get_value("")
        .ok()?;
    parse_quoted_executable(&command).filter(|p| p.exists())
}

#[cfg(windows)]
fn parse_quoted_executable(command: &str) -> Option<PathBuf> {
    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }
    command.split_whitespace().next().map(PathBuf::from)
}

fn read_offer_id(game_folder: &Path) -> Option<String> {
    for entry in fs::read_dir(game_folder).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("mfst") {
            continue;
        }
        let raw = fs::read_to_string(path).ok()?;
        let marker = "&id=";
        let start = raw.find(marker)? + marker.len();
        let end = raw[start..].find('&').map(|i| start + i)?;
        return Some(raw[start..end].to_string());
    }
    None
}
