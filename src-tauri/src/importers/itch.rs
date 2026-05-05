use crate::{
    error::{io_context, AppResult},
    importers::candidate_from_parts,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(app_data) = dirs::data_dir() else {
        return Ok(Vec::new());
    };
    let apps_dir = app_data.join("itch").join("apps");
    if !apps_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&apps_dir).map_err(io_context(&apps_dir))?.flatten() {
        let install_path = entry.path();
        if !install_path.is_dir() {
            continue;
        }
        let receipt = install_path.join(".itch").join("receipt.json");
        let title = read_itch_title(&receipt).unwrap_or_else(|| {
            install_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("itch.io game")
                .to_string()
        });
        let Some(executable_path) = find_launchable_file(&install_path) else {
            continue;
        };
        let start_dir = executable_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| install_path.clone());
        candidates.push(candidate_from_parts(
            user,
            ImportSource::Itch,
            "itch",
            title,
            executable_path,
            start_dir,
            None,
            vec!["itch.io".to_string()],
        ));
    }

    Ok(candidates)
}

fn read_itch_title(receipt: &Path) -> Option<String> {
    let raw = fs::read_to_string(receipt).ok()?;
    serde_json::from_str::<ItchReceipt>(&raw)
        .ok()
        .map(|receipt| receipt.game.title)
}

fn find_launchable_file(root: &Path) -> Option<PathBuf> {
    let preferred = ["exe", "bat", "cmd"];
    walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .flatten()
        .map(|entry| entry.path().to_path_buf())
        .find(|path| {
            path.is_file()
                && path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
                    preferred
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
        })
}

#[derive(Debug, Deserialize)]
struct ItchReceipt {
    game: ItchReceiptGame,
}

#[derive(Debug, Deserialize)]
struct ItchReceiptGame {
    title: String,
}
