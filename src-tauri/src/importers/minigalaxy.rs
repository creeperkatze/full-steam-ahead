use crate::{
    error::AppResult,
    importers::{candidate_from_parts, gog},
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
        // Windows installs keep goggame-*.info at the top level; Linux ones nest it under game/
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
        return None;
    }
    let name = std::fs::read_to_string(folder.join("gameinfo"))
        .ok()
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
    let raw = std::fs::read_to_string(config_file).ok()?;
    parse_install_dir(&raw)
}

fn parse_install_dir(raw: &str) -> Option<PathBuf> {
    serde_json::from_str::<MinigalaxyConfig>(raw)
        .ok()?
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
            parse_install_dir(raw),
            Some(PathBuf::from("/mnt/games/GOG"))
        );
    }

    #[test]
    fn ignores_missing_or_empty_install_dir() {
        assert_eq!(parse_install_dir(r#"{"locale":""}"#), None);
        assert_eq!(parse_install_dir(r#"{"install_dir":""}"#), None);
        assert_eq!(parse_install_dir("not json"), None);
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
