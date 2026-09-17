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
        tracing::warn!("LOCALAPPDATA is not set");
        return Ok(Vec::new());
    };

    let sqlite_path = amazon_root
        .join("Data")
        .join("Games")
        .join("Sql")
        .join("GameInstallInfo.sqlite");
    let launcher_path = amazon_root.join("App").join("Amazon Games.exe");
    for path in [&sqlite_path, &launcher_path] {
        if !path.exists() {
            tracing::debug!(path = %path.display(), "Amazon Games path not found");
            return Ok(Vec::new());
        }
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
    loop {
        match statement.next() {
            Ok(State::Row) => {}
            Ok(State::Done) => break,
            Err(error) => {
                tracing::warn!(%error, "Reading the Amazon Games database stopped early");
                break;
            }
        }
        let (id, title) = match (
            statement.read::<String, usize>(0),
            statement.read::<String, usize>(1),
        ) {
            (Ok(id), Ok(title)) => (id, title),
            (id, title) => {
                tracing::debug!(?id, ?title, "Skipping Amazon game with missing details");
                continue;
            }
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
