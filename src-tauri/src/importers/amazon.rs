use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use sqlite::State;
use std::path::PathBuf;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(sqlite_path) = local_app_data()
        .map(|path| {
            path.join("Amazon Games")
                .join("Data")
                .join("Games")
                .join("Sql")
                .join("GameInstallInfo.sqlite")
        })
        .filter(|path| path.exists())
    else {
        return Ok(Vec::new());
    };
    let Some(launcher_path) = local_app_data()
        .map(|path| {
            path.join("Amazon Games")
                .join("App")
                .join("Amazon Games.exe")
        })
        .filter(|path| path.exists())
    else {
        return Ok(Vec::new());
    };

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
