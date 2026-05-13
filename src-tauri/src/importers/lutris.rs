use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::process::Command;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let games = run_lutris_native()
        .or_else(|_| run_lutris_flatpak())
        .unwrap_or_default();

    let candidates = games
        .into_iter()
        .filter(|g| {
            // Exclude Steam games to avoid double-importing
            g.runner.as_deref() != Some("steam")
                && g.service.as_deref() != Some("steam")
        })
        .map(|game| {
            let (exe, opts) = lutris_launch_args(&game, false);
            launcher_candidate(
                user,
                ImportSource::Lutris,
                "lutris",
                game.name,
                exe.into(),
                opts,
                vec!["Lutris".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

fn lutris_launch_args(game: &LutrisGame, is_flatpak: bool) -> (String, String) {
    if is_flatpak {
        let flatpak_image = "net.lutris.Lutris";
        (
            "flatpak".to_string(),
            format!("run {} lutris:rungame/{}", flatpak_image, game.slug),
        )
    } else {
        ("lutris".to_string(), format!("lutris:rungame/{}", game.slug))
    }
}

fn run_lutris_native() -> Result<Vec<LutrisGame>, Box<dyn std::error::Error>> {
    let output = Command::new("lutris").args(["--json", "-lo"]).output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?)
}

fn run_lutris_flatpak() -> Result<Vec<LutrisGame>, Box<dyn std::error::Error>> {
    let output = Command::new("flatpak")
        .args(["run", "net.lutris.Lutris", "--json", "-lo"])
        .output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?)
}

#[derive(Deserialize, Clone)]
struct LutrisGame {
    slug: String,
    name: String,
    runner: Option<String>,
    service: Option<String>,
}
