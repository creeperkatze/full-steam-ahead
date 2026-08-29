use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use sqlite::State;
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let Some(amazon_root) = custom_path
        .map(PathBuf::from)
        .or_else(|| local_app_data().map(|path| path.join("Amazon Games")))
    else {
        return Ok(Vec::new());
    };

    let sqlite_path = amazon_root
        .join("Data")
        .join("Games")
        .join("Sql")
        .join("GameInstallInfo.sqlite");
    if !sqlite_path.exists() {
        return Ok(Vec::new());
    }
    let launcher_path = amazon_root.join("App").join("Amazon Games.exe");
    if !launcher_path.exists() {
        return Ok(Vec::new());
    }

    let connection = sqlite::open(&sqlite_path).map_err(|error| {
        AppError::Message(format!(
            "Could not read Amazon Games database at {}: {error}",
            sqlite_path.display()
        ))
    })?;
    let mut statement = connection
        .prepare("SELECT Id, ProductTitle FROM DbSet WHERE Installed = 1")
        .map_err(|error| AppError::Message(format!("Could not query Amazon Games: {error}")))?;

    let mut candidates = Vec::new();
    while let Ok(State::Row) = statement.next() {
        let Ok(id) = statement.read::<String, usize>(0) else {
            continue;
        };
        let Ok(title) = statement.read::<String, usize>(1) else {
            continue;
        };
        candidates.push(launcher_candidate(
            user,
            ImportSource::Amazon,
            "amazon",
            title,
            launcher_path.clone(),
            format!("amazon-games://play/{id}"),
            vec!["Amazon Games".to_string()],
        ));
    }

    Ok(candidates)
}

fn local_app_data() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
}
