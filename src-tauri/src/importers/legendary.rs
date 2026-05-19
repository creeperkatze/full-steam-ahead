use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::process::Command;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    // Try `legendary` first, then `rare` (alternative CLI)
    let games = run_legendary("legendary")
        .or_else(|_| run_legendary("rare"))
        .unwrap_or_default();

    let candidates = games
        .into_iter()
        .filter(|g| !g.is_dlc)
        .map(|game| {
            launcher_candidate(
                user,
                ImportSource::Legendary,
                "legendary",
                game.title,
                "legendary".into(),
                format!("launch {}", game.app_name),
                vec!["Legendary".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

fn run_legendary(executable: &str) -> Result<Vec<LegendaryGame>, Box<dyn std::error::Error>> {
    let output = Command::new(executable)
        .args(["list-installed", "--json"])
        .output()?;
    let json = String::from_utf8_lossy(&output.stdout);
    Ok(serde_json::from_str(&json)?)
}

#[derive(Deserialize)]
struct LegendaryGame {
    app_name: String,
    title: String,
    #[serde(default)]
    is_dlc: bool,
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
