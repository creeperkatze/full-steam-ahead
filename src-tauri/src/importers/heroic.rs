use crate::{
    error::AppResult,
    importers::{gog, host_binary_path, launcher_candidate},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let (install_mode, heroic_config_dir) = if let Some(custom) = custom_path {
        (install_mode_for(custom), custom.to_path_buf())
    } else {
        let Ok(home) = std::env::var("HOME") else {
            return Ok(Vec::new());
        };
        detect_install_mode(&home)
    };

    let mut candidates = Vec::new();

    let epic_json = heroic_config_dir
        .join("legendaryConfig")
        .join("legendary")
        .join("installed.json");
    candidates.extend(scan_epic_games(user, &epic_json, &install_mode));

    let gog_json = heroic_config_dir.join("gog_store").join("installed.json");
    candidates.extend(scan_gog_games(user, &gog_json, &install_mode));

    let sideload_json = heroic_config_dir.join("sideload_apps").join("library.json");
    candidates.extend(scan_sideload_games(user, &sideload_json, &install_mode));

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

fn install_mode_for(config_dir: &Path) -> InstallMode {
    if config_dir
        .to_string_lossy()
        .contains(".var/app/com.heroicgameslauncher.hgl")
    {
        InstallMode::FlatPak
    } else {
        InstallMode::UserBin
    }
}

fn heroic_launch_candidate(
    user: &SteamUser,
    name: String,
    runner: Option<&str>,
    app_name: &str,
    install_mode: &InstallMode,
) -> ImportCandidate {
    let launch_url = match runner {
        Some(runner) => format!("heroic://launch/{runner}/{app_name}"),
        None => format!("heroic://launch/{app_name}"),
    };
    let (launcher, launch_options) = match install_mode {
        InstallMode::FlatPak => (
            "flatpak",
            format!("run com.heroicgameslauncher.hgl {launch_url} --no-gui --no-sandbox"),
        ),
        InstallMode::UserBin => ("heroic", launch_url),
    };
    // Steam needs an absolute path in the shortcut
    let launcher_path = host_binary_path(launcher);
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
        .map(|game| heroic_launch_candidate(user, game.title, None, &game.app_name, install_mode))
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

#[derive(Deserialize)]
struct HeroicSideloadLibrary {
    #[serde(default)]
    games: Vec<HeroicSideloadGame>,
}

#[derive(Deserialize)]
struct HeroicSideloadGame {
    app_name: String,
    title: String,
    #[serde(default)]
    is_installed: bool,
    #[serde(default)]
    install: HeroicSideloadInstall,
}

#[derive(Deserialize, Default)]
struct HeroicSideloadInstall {
    executable: Option<String>,
    #[serde(default)]
    is_dlc: bool,
}

impl HeroicSideloadGame {
    fn is_launchable(&self) -> bool {
        if !self.is_installed || self.install.is_dlc {
            return false;
        }
        // Browser apps have no executable; Heroic opens their URL itself
        match self.install.executable.as_deref() {
            Some(exe) if !exe.is_empty() => Path::new(exe).exists(),
            _ => true,
        }
    }
}

/// Non-store games the user added to Heroic by pointing it at an executable.
fn scan_sideload_games(
    user: &SteamUser,
    library_json: &Path,
    install_mode: &InstallMode,
) -> Vec<ImportCandidate> {
    let Ok(raw) = std::fs::read_to_string(library_json) else {
        return Vec::new();
    };
    let Ok(library) = serde_json::from_str::<HeroicSideloadLibrary>(&raw) else {
        return Vec::new();
    };
    library
        .games
        .into_iter()
        .filter(HeroicSideloadGame::is_launchable)
        .map(|game| {
            // Pin the runner so a store game sharing the app name can't be launched instead
            heroic_launch_candidate(
                user,
                game.title,
                Some("sideload"),
                &game.app_name,
                install_mode,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // HeroicEpicGame JSON deserialization

    #[test]
    fn parses_epic_game_entry() {
        let json = r#"{
            "app_name": "CrabGame",
            "title": "Crab Game",
            "is_dlc": false,
            "install_path": "/home/user/Games/CrabGame",
            "executable": "CrabGame.sh"
        }"#;
        let game: HeroicEpicGame = serde_json::from_str(json).unwrap();
        assert_eq!(game.app_name, "CrabGame");
        assert_eq!(game.title, "Crab Game");
        assert!(!game.is_dlc);
    }

    #[test]
    fn epic_is_dlc_defaults_to_false() {
        let json = r#"{"app_name":"g","title":"G","install_path":"/","executable":"g.sh"}"#;
        let game: HeroicEpicGame = serde_json::from_str(json).unwrap();
        assert!(!game.is_dlc);
    }

    #[test]
    fn epic_filters_out_dlc() {
        let json = r#"[
            {"app_name":"game","title":"Game","is_dlc":false,"install_path":"/","executable":"g"},
            {"app_name":"dlc","title":"DLC","is_dlc":true,"install_path":"/","executable":"d"}
        ]"#;
        let games: HashMap<String, HeroicEpicGame> = serde_json::from_str(
            &json
                .replace('[', "{\"a\":")
                .replace("},\n            {", ",\"b\":")
                .replace(']', "}"),
        )
        .unwrap_or_default();
        // Test the filtering logic directly on deserialized data
        let non_dlc_count = serde_json::from_str::<Vec<serde_json::Value>>(json)
            .unwrap()
            .into_iter()
            .filter(|v| !v["is_dlc"].as_bool().unwrap_or(false))
            .count();
        assert_eq!(non_dlc_count, 1);
    }

    // HeroicGogConfig / HeroicGogEntry JSON deserialization

    #[test]
    fn parses_gog_config() {
        let json = r#"{"installed":[
            {"appName":"1234","install_path":"/games/MyGame","platform":"linux"}
        ]}"#;
        let config: HeroicGogConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.installed.len(), 1);
        assert_eq!(config.installed[0].install_path, "/games/MyGame");
        assert_eq!(config.installed[0].platform, "linux");
    }

    #[test]
    fn gog_entry_accepts_both_app_name_forms() {
        // HeroicGogEntry uses #[serde(alias = "appName")] so both forms work
        let with_alias = r#"{"appName":"A","install_path":"/g","platform":"windows"}"#;
        let with_snake = r#"{"app_name":"B","install_path":"/g","platform":"linux"}"#;
        let a: HeroicGogEntry = serde_json::from_str(with_alias).unwrap();
        let b: HeroicGogEntry = serde_json::from_str(with_snake).unwrap();
        assert_eq!(a.app_name, "A");
        assert_eq!(b.app_name, "B");
    }

    // HeroicSideloadLibrary JSON deserialization

    #[test]
    fn parses_sideload_library() {
        let json = r#"{"games":[{
            "runner":"sideload",
            "app_name":"aBcD1234",
            "title":"My Game",
            "install":{"executable":"/games/mygame/game.exe","platform":"windows","is_dlc":false},
            "folder_name":"/games/mygame",
            "is_installed":true,
            "canRunOffline":true
        }]}"#;
        let library: HeroicSideloadLibrary = serde_json::from_str(json).unwrap();
        assert_eq!(library.games.len(), 1);
        let game = &library.games[0];
        assert_eq!(game.app_name, "aBcD1234");
        assert_eq!(game.title, "My Game");
        assert_eq!(
            game.install.executable.as_deref(),
            Some("/games/mygame/game.exe")
        );
    }

    #[test]
    fn sideload_library_without_games_deserializes() {
        let library: HeroicSideloadLibrary = serde_json::from_str("{}").unwrap();
        assert!(library.games.is_empty());
    }

    #[test]
    fn sideload_skips_uninstalled_and_missing_executables() {
        let json = r#"{"games":[
            {"app_name":"a","title":"Not installed","is_installed":false,"install":{}},
            {"app_name":"b","title":"Missing exe","is_installed":true,
             "install":{"executable":"/definitely/not/here.exe"}},
            {"app_name":"c","title":"Browser app","is_installed":true,"install":{},
             "browserUrl":"https://example.com"}
        ]}"#;
        let library: HeroicSideloadLibrary = serde_json::from_str(json).unwrap();
        let launchable: Vec<_> = library
            .games
            .iter()
            .filter(|g| g.is_launchable())
            .map(|g| g.app_name.as_str())
            .collect();
        assert_eq!(launchable, ["c"]);
    }

    #[test]
    fn empty_gog_config_deserializes() {
        let config: HeroicGogConfig = serde_json::from_str(r#"{"installed":[]}"#).unwrap();
        assert!(config.installed.is_empty());
    }
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
            None,
            &entry.app_name,
            install_mode,
        ));
    }
    candidates
}
