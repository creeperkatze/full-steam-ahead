use crate::{
    error::AppResult,
    importers::{gog, launcher_candidate},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Ok(home) = std::env::var("HOME") else {
        return Ok(Vec::new());
    };

    let (install_mode, heroic_config_dir) = detect_install_mode(&home);

    let mut candidates = Vec::new();

    let epic_json = heroic_config_dir
        .join("legendaryConfig")
        .join("legendary")
        .join("installed.json");
    candidates.extend(scan_epic_games(user, &epic_json, &install_mode));

    let gog_json = heroic_config_dir.join("gog_store").join("installed.json");
    candidates.extend(scan_gog_games(user, &gog_json, &install_mode));

    Ok(candidates)
}

enum InstallMode {
    FlatPak,
    UserBin,
}

fn detect_install_mode(home: &str) -> (InstallMode, PathBuf) {
    let flatpak_config = PathBuf::from(home)
        .join(".var")
        .join("app")
        .join("com.heroicgameslauncher.hgl")
        .join("config")
        .join("heroic");
    if flatpak_config.exists() {
        return (InstallMode::FlatPak, flatpak_config);
    }
    let user_bin_config = PathBuf::from(home).join(".config").join("heroic");
    (InstallMode::UserBin, user_bin_config)
}

fn heroic_launch_candidate(
    user: &SteamUser,
    name: String,
    app_name: &str,
    install_mode: &InstallMode,
) -> ImportCandidate {
    let launch_url = format!("heroic://launch/{app_name}");
    let (launcher_path, launch_options) = match install_mode {
        InstallMode::FlatPak => (
            PathBuf::from("flatpak"),
            format!("run com.heroicgameslauncher.hgl {launch_url} --no-gui --no-sandbox"),
        ),
        InstallMode::UserBin => (PathBuf::from("heroic"), launch_url),
    };
    launcher_candidate(
        user,
        ImportSource::Heroic,
        "heroic",
        name,
        launcher_path,
        launch_options,
        vec!["Heroic".to_string()],
    )
}

#[derive(Deserialize)]
struct HeroicEpicGame {
    app_name: String,
    title: String,
    #[serde(default)]
    is_dlc: bool,
    install_path: String,
    executable: String,
}

impl HeroicEpicGame {
    fn is_installed(&self) -> bool {
        Path::new(&self.install_path)
            .join(&self.executable)
            .exists()
    }
}

fn scan_epic_games(
    user: &SteamUser,
    installed_json: &Path,
    install_mode: &InstallMode,
) -> Vec<ImportCandidate> {
    let Ok(raw) = std::fs::read_to_string(installed_json) else {
        return Vec::new();
    };
    let Ok(map) = serde_json::from_str::<HashMap<String, HeroicEpicGame>>(&raw) else {
        return Vec::new();
    };
    map.into_values()
        .filter(|g| !g.is_dlc && g.is_installed())
        .map(|game| heroic_launch_candidate(user, game.title, &game.app_name, install_mode))
        .collect()
}

#[derive(Deserialize)]
struct HeroicGogConfig {
    installed: Vec<HeroicGogEntry>,
}

#[derive(Deserialize)]
struct HeroicGogEntry {
    #[serde(alias = "appName")]
    app_name: String,
    install_path: String,
    platform: String,
}

fn scan_gog_games(
    user: &SteamUser,
    installed_json: &Path,
    install_mode: &InstallMode,
) -> Vec<ImportCandidate> {
    let Ok(raw) = std::fs::read_to_string(installed_json) else {
        return Vec::new();
    };
    let Ok(config) = serde_json::from_str::<HeroicGogConfig>(&raw) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for entry in config.installed {
        let install_path = PathBuf::from(&entry.install_path);
        if !install_path.exists() {
            continue;
        }

        if entry.platform == "linux" {
            let gog_candidates = gog::scan_folders(user, vec![install_path.clone()]);
            if !gog_candidates.is_empty() {
                for mut c in gog_candidates {
                    c.source = ImportSource::Heroic;
                    c.id = c.id.replace("gog-", "heroic-");
                    c.tags = vec!["Heroic".to_string(), "GOG".to_string()];
                    candidates.push(c);
                }
                continue;
            }
        }

        let name = install_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&entry.app_name)
            .to_string();
        candidates.push(heroic_launch_candidate(
            user,
            name,
            &entry.app_name,
            install_mode,
        ));
    }
    candidates
}
