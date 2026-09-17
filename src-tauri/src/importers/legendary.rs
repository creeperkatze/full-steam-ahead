use crate::{
    error::AppResult,
    importers::{host_binary_path, host_command, launcher_candidate},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let (executable, games) = if let Some(custom) = custom_path {
        // A custom executable was explicitly chosen, so don't fall back to Rare
        let exe = custom.to_string_lossy().to_string();
        let games = run_legendary(&exe).unwrap_or_default();
        (exe, games)
    } else if let Ok(games) = run_legendary("legendary") {
        ("legendary".to_string(), games)
    } else if super::resolve_host_binary("rare").is_some() {
        // Rare is a GUI without `list-installed`, but it shares Legendary's config
        // and can launch games itself
        let games = read_installed_json(&config_dir()).unwrap_or_default();
        ("rare".to_string(), games)
    } else {
        return Ok(Vec::new());
    };

    let candidates = games
        .into_iter()
        .filter(|g| !g.is_dlc)
        .map(|game| {
            launcher_candidate(
                user,
                ImportSource::Legendary,
                "legendary",
                game.title,
                host_binary_path(&executable),
                format!("launch {}", game.app_name),
                vec!["Legendary".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

fn run_legendary(executable: &str) -> Result<Vec<LegendaryGame>, Box<dyn std::error::Error>> {
    let output = host_command(executable)
        .args(["list-installed", "--json"])
        .output()?;
    let json = String::from_utf8_lossy(&output.stdout);
    Ok(serde_json::from_str(&json)?)
}

/// Matches how Legendary resolves its config directory.
fn config_dir() -> PathBuf {
    if let Ok(path) = std::env::var("LEGENDARY_CONFIG_PATH") {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(path).join("legendary");
    }
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config")
        .join("legendary")
}

fn read_installed_json(config_dir: &Path) -> Option<Vec<LegendaryGame>> {
    let raw = std::fs::read_to_string(config_dir.join("installed.json")).ok()?;
    parse_installed_json(&raw)
}

fn parse_installed_json(raw: &str) -> Option<Vec<LegendaryGame>> {
    let map = serde_json::from_str::<HashMap<String, LegendaryGame>>(raw).ok()?;
    Some(
        map.into_values()
            .filter(|g| {
                g.install_path
                    .as_deref()
                    .is_none_or(|p| Path::new(p).exists())
            })
            .collect(),
    )
}

#[derive(Deserialize)]
struct LegendaryGame {
    app_name: String,
    title: String,
    #[serde(default)]
    is_dlc: bool,
    install_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_game_list() {
        let json = r#"[{"app_name":"game1","title":"Game One","is_dlc":false}]"#;
        let games: Vec<LegendaryGame> = serde_json::from_str(json).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_name, "game1");
        assert_eq!(games[0].title, "Game One");
    }

    #[test]
    fn installed_json_skips_missing_install_paths() {
        let json = r#"{
            "Kept": {"app_name":"Kept","title":"Kept","version":"1","install_path":"/"},
            "Gone": {"app_name":"Gone","title":"Gone","version":"1","install_path":"/definitely/not/here"}
        }"#;
        let games = parse_installed_json(json).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_name, "Kept");
    }

    #[test]
    fn is_dlc_defaults_to_false_when_absent() {
        let json = r#"[{"app_name":"g","title":"G"}]"#;
        let games: Vec<LegendaryGame> = serde_json::from_str(json).unwrap();
        assert!(!games[0].is_dlc);
    }

    #[test]
    fn filters_out_dlc_entries() {
        let json = r#"[
            {"app_name":"game","title":"Game","is_dlc":false},
            {"app_name":"dlc","title":"DLC Pack","is_dlc":true}
        ]"#;
        let games: Vec<LegendaryGame> = serde_json::from_str(json).unwrap();
        let non_dlc: Vec<_> = games.into_iter().filter(|g| !g.is_dlc).collect();
        assert_eq!(non_dlc.len(), 1);
        assert_eq!(non_dlc[0].app_name, "game");
    }

    #[test]
    fn empty_list_deserializes() {
        let games: Vec<LegendaryGame> = serde_json::from_str("[]").unwrap();
        assert!(games.is_empty());
    }

    #[test]
    fn invalid_json_is_handled_gracefully() {
        let result = serde_json::from_str::<Vec<LegendaryGame>>("not json");
        assert!(result.is_err());
    }
}
