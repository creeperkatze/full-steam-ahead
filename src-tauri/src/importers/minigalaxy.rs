use crate::{
    error::AppResult,
    importers::{candidate_from_parts, gog, read_launcher_file, read_launcher_json},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let games_dir = match custom_path {
        Some(p) => p.to_path_buf(),
        None => default_games_dir()?,
    };
    if !games_dir.exists() {
        tracing::debug!(path = %games_dir.display(), "Minigalaxy games folder not found");
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

    let mut candidates = Vec::new();
    for folder in game_folders {
        // Only Windows installs keep goggame-*.info at the top level.
        let windows = gog::scan_folders(user, vec![folder.clone()]);
        if windows.is_empty() {
            candidates.extend(native_candidate(user, &folder));
            continue;
        }
        for mut c in windows {
            c.source = ImportSource::MiniGalaxy;
            c.id = c.id.replace("gog-", "minigalaxy-");
            c.tags = tags();
            // Windows games need Proton when started from Steam
            c.needs_proton = true;
            candidates.push(c);
        }
    }

    Ok(candidates)
}

fn tags() -> Vec<String> {
    vec!["MiniGalaxy".to_string(), "GOG".to_string()]
}

/// Linux-native GOG installs, which Minigalaxy starts through their `start.sh`.
fn native_candidate(user: &SteamUser, folder: &Path) -> Option<ImportCandidate> {
    let start_script = folder.join("start.sh");
    if !start_script.is_file() {
        tracing::debug!(path = %folder.display(), "Skipping Minigalaxy folder without a game");
        return None;
    }
    let name = read_launcher_file(&folder.join("gameinfo"))
        .and_then(|info| game_name_from_gameinfo(&info))
        .or_else(|| folder.file_name()?.to_str().map(str::to_string))?;
    Some(candidate_from_parts(
        user,
        ImportSource::MiniGalaxy,
        "minigalaxy",
        name,
        start_script,
        folder.to_path_buf(),
        None,
        tags(),
    ))
}

/// The first line of GOG's Linux `gameinfo` file is the game's name.
fn game_name_from_gameinfo(info: &str) -> Option<String> {
    info.lines()
        .next()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

#[derive(Deserialize)]
struct MinigalaxyConfig {
    install_dir: Option<String>,
}

fn default_games_dir() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| {
        crate::error::AppError::Message("HOME environment variable not set".to_string())
    })?;
    let home = PathBuf::from(home);

    let native_config = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));
    let flatpak_config = home
        .join(".var")
        .join("app")
        .join("io.github.sharkwouter.Minigalaxy")
        .join("config");

    let configured = [native_config, flatpak_config]
        .into_iter()
        .find_map(|dir| install_dir_from_config(&dir.join("minigalaxy").join("config.json")));
    Ok(configured.unwrap_or_else(|| home.join("GOG Games")))
}

fn install_dir_from_config(config_file: &Path) -> Option<PathBuf> {
    install_dir(read_launcher_json(config_file)?)
}

fn install_dir(config: MinigalaxyConfig) -> Option<PathBuf> {
    config
        .install_dir
        .filter(|dir| !dir.trim().is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_install_dir_from_config() {
        let raw = r#"{"locale":"","install_dir":"/mnt/games/GOG","keep_installers":false}"#;
        assert_eq!(
            install_dir(serde_json::from_str(raw).unwrap()),
            Some(PathBuf::from("/mnt/games/GOG"))
        );
    }

    #[test]
    fn ignores_missing_or_empty_install_dir() {
        let parse = |raw| install_dir(serde_json::from_str(raw).unwrap());
        assert_eq!(parse(r#"{"locale":""}"#), None);
        assert_eq!(parse(r#"{"install_dir":""}"#), None);
    }

    #[test]
    fn gameinfo_name_is_the_first_line() {
        assert_eq!(
            game_name_from_gameinfo("Stardew Valley\n1.6.15\n").as_deref(),
            Some("Stardew Valley")
        );
        assert_eq!(game_name_from_gameinfo("\n1.0\n"), None);
        assert_eq!(game_name_from_gameinfo(""), None);
    }
}
