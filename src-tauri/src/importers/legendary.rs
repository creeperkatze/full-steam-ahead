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
