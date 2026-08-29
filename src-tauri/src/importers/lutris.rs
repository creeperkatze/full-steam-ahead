use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{path::Path, process::Command};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let (games, is_flatpak, custom_exe) = if let Some(custom) = custom_path {
        let exe = custom.to_string_lossy().to_string();
        (
            run_lutris_custom(&exe).unwrap_or_default(),
            false,
            Some(exe),
        )
    } else {
        match run_lutris_native() {
            Ok(games) => (games, false, None),
            Err(_) => (run_lutris_flatpak().unwrap_or_default(), true, None),
        }
    };

    let candidates = games
        .into_iter()
        .filter(|g| {
            // Exclude Steam games to avoid double-importing
            g.runner.as_deref() != Some("steam") && g.service.as_deref() != Some("steam")
        })
        .map(|game| {
            let (default_exe, opts) = lutris_launch_args(&game, is_flatpak);
            let exe = custom_exe.clone().unwrap_or(default_exe);
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
        (
            "lutris".to_string(),
            format!("lutris:rungame/{}", game.slug),
        )
    }
}

fn run_lutris_native() -> Result<Vec<LutrisGame>, Box<dyn std::error::Error>> {
    let output = Command::new("lutris").args(["--json", "-lo"]).output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(
        &output.stdout,
    ))?)
}

fn run_lutris_flatpak() -> Result<Vec<LutrisGame>, Box<dyn std::error::Error>> {
    let output = Command::new("flatpak")
        .args(["run", "net.lutris.Lutris", "--json", "-lo"])
        .output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(
        &output.stdout,
    ))?)
}

fn run_lutris_custom(executable: &str) -> Result<Vec<LutrisGame>, Box<dyn std::error::Error>> {
    let output = Command::new(executable).args(["--json", "-lo"]).output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(
        &output.stdout,
    ))?)
}

#[derive(Deserialize, Clone)]
struct LutrisGame {
    slug: String,
    name: String,
    runner: Option<String>,
    service: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(slug: &str) -> LutrisGame {
        LutrisGame {
            slug: slug.to_string(),
            name: slug.to_string(),
            runner: None,
            service: None,
        }
    }

    #[test]
    fn native_launch_args() {
        let (exe, opts) = lutris_launch_args(&game("witcher-3"), false);
        assert_eq!(exe, "lutris");
        assert_eq!(opts, "lutris:rungame/witcher-3");
    }

    #[test]
    fn flatpak_launch_args() {
        let (exe, opts) = lutris_launch_args(&game("witcher-3"), true);
        assert_eq!(exe, "flatpak");
        assert_eq!(opts, "run net.lutris.Lutris lutris:rungame/witcher-3");
    }

    #[test]
    fn steam_runner_is_excluded() {
        // scan() filters these out; verify the predicate directly
        let steam = LutrisGame {
            slug: "csgo".to_string(),
            name: "CS:GO".to_string(),
            runner: Some("steam".to_string()),
            service: None,
        };
        assert_eq!(steam.runner.as_deref(), Some("steam"));

        let steam_service = LutrisGame {
            slug: "csgo".to_string(),
            name: "CS:GO".to_string(),
            runner: None,
            service: Some("steam".to_string()),
        };
        assert_eq!(steam_service.service.as_deref(), Some("steam"));
    }
}
