use crate::{
    error::{io_context, AppResult},
    importers::candidate_from_parts,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::{fs, path::PathBuf};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(data_dir) = dirs::data_dir() else {
        return Ok(Vec::new());
    };
    let games_dir = data_dir.join("Playnite").join("library").join("games");
    if !games_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&games_dir).map_err(io_context(&games_dir))? {
        let entry = entry.map_err(io_context(&games_dir))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };

        let Some(name) = value.get("Name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(action) = value
            .get("PlayAction")
            .and_then(|value| value.get("Path"))
            .and_then(|value| value.as_str())
        else {
            continue;
        };

        let executable_path = PathBuf::from(action);
        let start_dir = executable_path.parent().map(PathBuf::from).unwrap_or_default();
        candidates.push(candidate_from_parts(
            user,
            ImportSource::Playnite,
            "playnite",
            name.to_string(),
            executable_path,
            start_dir,
            None,
            vec!["Playnite".to_string()],
        ));
    }

    Ok(candidates)
}
