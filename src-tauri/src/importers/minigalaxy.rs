use crate::{
    error::AppResult,
    importers::gog,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::path::PathBuf;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let games_dir = default_games_dir()?;
    if !games_dir.exists() {
        return Ok(Vec::new());
    }

    let game_folders: Vec<PathBuf> = std::fs::read_dir(&games_dir)
        .map_err(|e| crate::error::AppError::Io {
            path: games_dir,
            source: e,
        })?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    let mut candidates = gog::scan_folders(user, game_folders);

    for c in &mut candidates {
        c.source = ImportSource::MiniGalaxy;
        c.id = c.id.replace("gog-", "minigalaxy-");
        c.tags = vec!["MiniGalaxy".to_string(), "GOG".to_string()];
    }

    Ok(candidates)
}

fn default_games_dir() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| {
        crate::error::AppError::Message("HOME environment variable not set".to_string())
    })?;
    Ok(PathBuf::from(home).join("GOG Games"))
}
