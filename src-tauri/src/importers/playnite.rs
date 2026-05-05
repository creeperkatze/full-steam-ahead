use crate::{
    error::{io_context, AppResult},
    importers::quote_path,
    models::{ImportCandidate, ImportSource, SteamUser},
    steam::{artwork, non_steam_app_id},
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
        let app_id = non_steam_app_id(&quote_path(&executable_path), name);
        let (matched_steam_app_id, artwork) =
            artwork::steam_preferred_plan(&user.grid_path, app_id, name);
        candidates.push(ImportCandidate {
            id: format!("playnite-{app_id}"),
            source: ImportSource::Playnite,
            name: name.to_string(),
            start_dir: executable_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_default(),
            executable_path,
            launch_options: None,
            existing_app_id: None,
            matched_steam_app_id,
            tags: vec!["Playnite".to_string()],
            artwork,
        });
    }

    Ok(candidates)
}
