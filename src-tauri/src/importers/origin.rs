use crate::{
    error::{io_context, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let origin_folder = program_data().join("Origin");
    let local_content = origin_folder.join("LocalContent");
    if !local_content.exists() {
        return Ok(Vec::new());
    }
    let Some(launcher_path) = origin_launcher_path() else {
        return Ok(Vec::new());
    };

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&local_content).map_err(io_context(&local_content))?.flatten() {
        let game_folder = entry.path();
        if !game_folder.is_dir() {
            continue;
        }
        let Some(id) = read_origin_offer_id(&game_folder) else {
            continue;
        };
        let title = game_folder
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("EA game")
            .to_string();
        candidates.push(launcher_candidate(
            user,
            ImportSource::Origin,
            "origin",
            title,
            launcher_path.clone(),
            format!("origin2://game/launch?offerIds={id}&autoDownload=1&authCode=&cmdParams="),
            vec!["EA app / Origin".to_string()],
        ));
    }

    Ok(candidates)
}

fn program_data() -> PathBuf {
    std::env::var("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
}

#[cfg(windows)]
fn origin_launcher_path() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};

    let command: String = RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey("eadm\\shell\\open\\command")
        .ok()?
        .get_value("")
        .ok()?;
    parse_quoted_executable(&command).filter(|path| path.exists())
}

#[cfg(not(windows))]
fn origin_launcher_path() -> Option<PathBuf> {
    None
}

fn parse_quoted_executable(command: &str) -> Option<PathBuf> {
    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }
    command.split_whitespace().next().map(PathBuf::from)
}

fn read_origin_offer_id(game_folder: &Path) -> Option<String> {
    for entry in fs::read_dir(game_folder).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("mfst") {
            continue;
        }
        let raw = fs::read_to_string(path).ok()?;
        let marker = "&id=";
        let start = raw.find(marker)? + marker.len();
        let end = raw[start..].find('&').map(|index| start + index)?;
        return Some(raw[start..end].to_string());
    }
    None
}
