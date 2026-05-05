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
    let galaxy_path = program_data()
        .join("GOG.com")
        .join("Galaxy")
        .join("config.json");
    if !galaxy_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&galaxy_path).map_err(io_context(&galaxy_path))?;
    let Ok(config) = serde_json::from_str::<GogConfig>(&raw) else {
        return Ok(Vec::new());
    };

    let mut roots = config.installation_paths.unwrap_or_default();
    if roots.is_empty() {
        if let Some(path) = config.library_path {
            roots.push(path);
        }
    }

    let mut candidates = Vec::new();
    for root in roots {
        let root = PathBuf::from(root);
        if !root.exists() {
            continue;
        }
        for entry in fs::read_dir(&root).map_err(io_context(&root))?.flatten() {
            let folder = entry.path();
            if !folder.is_dir() {
                continue;
            }
            let game_folder = if folder.join("game").exists() {
                folder.join("game")
            } else {
                folder
            };
            candidates.extend(scan_gog_folder(user, &game_folder)?);
        }
    }

    Ok(candidates)
}

fn scan_gog_folder(user: &SteamUser, game_folder: &Path) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(game_folder).map_err(io_context(game_folder))?.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("goggame-")
            || path.extension().and_then(|ext| ext.to_str()) != Some("info")
        {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
        let Ok(game) = serde_json::from_str::<GogGame>(&raw) else {
            continue;
        };
        let Some(task) = game.play_tasks.unwrap_or_default().into_iter().find(|task| {
            task.is_primary.unwrap_or_default()
                && task.task_type == "FileTask"
                && matches!(task.category.as_deref(), Some("launcher") | Some("game"))
                && task.path.is_some()
        }) else {
            continue;
        };
        let executable_path = game_folder.join(task.path.unwrap_or_default());
        let start_dir = task
            .working_dir
            .map(|dir| game_folder.join(dir))
            .unwrap_or_else(|| game_folder.to_path_buf());
        candidates.push(candidate_from_parts(
            user,
            ImportSource::Gog,
            "gog",
            game.name,
            executable_path,
            start_dir,
            task.arguments.filter(|args| !args.trim().is_empty()),
            vec!["GOG".to_string()],
        ));
    }
    Ok(candidates)
}

fn program_data() -> PathBuf {
    std::env::var("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogConfig {
    installation_paths: Option<Vec<String>>,
    library_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogGame {
    name: String,
    play_tasks: Option<Vec<GogPlayTask>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogPlayTask {
    category: Option<String>,
    is_primary: Option<bool>,
    path: Option<String>,
    #[serde(rename = "type")]
    task_type: String,
    working_dir: Option<String>,
    arguments: Option<String>,
}
