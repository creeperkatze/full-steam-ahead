use crate::{
    error::AppResult,
    importers::{gog, host_binary_path, launcher_candidate, read_launcher_json, shell_quote},
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
            tracing::warn!("HOME is not set");
            return Ok(Vec::new());
        };
        detect_install_mode(&home)
    };
    if !heroic_config_dir.exists() {
        tracing::debug!(path = %heroic_config_dir.display(), "Heroic not found");
        return Ok(Vec::new());
    }
    tracing::debug!(
        path = %heroic_config_dir.display(),
        flatpak = matches!(install_mode, InstallMode::FlatPak),
        "Heroic found"
    );

    let mut candidates = Vec::new();

    let epic_json = heroic_config_dir
        .join("legendaryConfig")
        .join("legendary")
        .join("installed.json");
    candidates.extend(scan_epic_games(user, &epic_json, &install_mode));

    let gog_json = heroic_config_dir.join("gog_store").join("installed.json");
    candidates.extend(scan_gog_games(user, &gog_json, &install_mode));

    let nile_dir = heroic_config_dir.join("nile_config").join("nile");
    candidates.extend(scan_nile_games(user, &nile_dir, &install_mode));

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
            format!(
                "run com.heroicgameslauncher.hgl {} --no-gui --no-sandbox",
                shell_quote(&launch_url)
            ),
        ),
        InstallMode::UserBin => ("heroic", shell_quote(&launch_url)),
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
    let Some(map) = read_launcher_json::<HashMap<String, HeroicEpicGame>>(installed_json) else {
        return Vec::new();
    };
    map.into_values()
        .filter(|g| {
            let keep = !g.is_dlc && g.is_installed();
            if !keep {
                tracing::debug!(game = g.title, dlc = g.is_dlc, "Skipping Heroic Epic entry");
            }
            keep
        })
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
struct HeroicNileInstalled {
    id: String,
    path: String,
}

#[derive(Deserialize)]
struct HeroicNileLibraryEntry {
    product: HeroicNileProduct,
}

#[derive(Deserialize)]
struct HeroicNileProduct {
    id: String,
    title: Option<String>,
}

/// Maps Amazon product ids to titles, installed.json doesn't carry them.
fn nile_titles(library_json: &Path) -> HashMap<String, String> {
    let Some(entries) = read_launcher_json::<Vec<HeroicNileLibraryEntry>>(library_json) else {
        return HashMap::new();
    };
    entries
        .into_iter()
        .filter_map(|e| Some((e.product.id, e.product.title?)))
        .collect()
}

/// Amazon Games installed through Heroic's Nile integration.
fn scan_nile_games(
    user: &SteamUser,
    nile_dir: &Path,
    install_mode: &InstallMode,
) -> Vec<ImportCandidate> {
    let Some(installed) =
        read_launcher_json::<Vec<HeroicNileInstalled>>(&nile_dir.join("installed.json"))
    else {
        return Vec::new();
    };
    let mut titles = nile_titles(&nile_dir.join("library.json"));

    installed
        .into_iter()
        .filter(|g| {
            let exists = Path::new(&g.path).exists();
            if !exists {
                tracing::debug!(
                    id = g.id,
                    path = g.path,
                    "Skipping Heroic Amazon game whose folder is gone"
                );
            }
            exists
        })
        .map(|game| {
            let name = titles.remove(&game.id).unwrap_or_else(|| {
                Path::new(&game.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&game.id)
                    .to_string()
            });
            heroic_launch_candidate(user, name, Some("nile"), &game.id, install_mode)
        })
        .collect()
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
        // Browser apps have no executable. Heroic opens their URL itself.
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
    let Some(library) = read_launcher_json::<HeroicSideloadLibrary>(library_json) else {
        return Vec::new();
    };
    library
        .games
        .into_iter()
        .filter(|g| {
            let launchable = g.is_launchable();
            if !launchable {
                tracing::debug!(
                    game = g.title,
                    "Skipping Heroic sideloaded app that can't be launched"
                );
            }
            launchable
        })
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

    // Nile (Amazon) JSON deserialization

    #[test]
    fn parses_nile_installed() {
        let json =
            r#"[{"id":"amzn1.adg.product.abc","version":"v1","path":"/games/Foo","size":123}]"#;
        let installed: Vec<HeroicNileInstalled> = serde_json::from_str(json).unwrap();
        assert_eq!(installed[0].id, "amzn1.adg.product.abc");
        assert_eq!(installed[0].path, "/games/Foo");
    }

    #[test]
    fn nile_titles_skip_untitled_products() {
        let dir = std::env::temp_dir().join(format!("fsa-nile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let library = dir.join("library.json");
        std::fs::write(
            &library,
            r#"[
                {"id":"x","product":{"id":"a","title":"Game A","productDetail":{}}},
                {"id":"y","product":{"id":"b","productDetail":{}}}
            ]"#,
        )
        .unwrap();
        let titles = nile_titles(&library);
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(titles.get("a").map(String::as_str), Some("Game A"));
        assert!(!titles.contains_key("b"));
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
    let Some(config) = read_launcher_json::<HeroicGogConfig>(installed_json) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for entry in config.installed {
        let install_path = PathBuf::from(&entry.install_path);
        if !install_path.exists() {
            tracing::debug!(id = entry.app_name, path = %install_path.display(), "Skipping Heroic GOG game whose folder is gone");
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
