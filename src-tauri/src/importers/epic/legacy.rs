//! The original Epic Games Launcher keeps one manifest per installed game in its data folder.

use super::{EpicGame, EpicLauncher, EpicLibrary};
use crate::{
    error::{io_context, AppResult},
    importers::read_launcher_json,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct LegacyPaths {
    launcher: EpicLauncher,
    manifest_folder_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Manifest {
    launch_executable: String,
    manifest_location: String,
    display_name: String,
    install_location: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    #[serde(rename = "bIsManaged")]
    is_managed: bool,
    #[serde(rename = "bRequiresAuth", default)]
    requires_auth: bool,
    #[serde(rename = "ExpectingDLCInstalled")]
    expected_dlc: Option<HashMap<String, bool>>,
}

impl Manifest {
    fn is_installed(&self) -> bool {
        Path::new(&self.manifest_location).exists()
    }

    fn is_launchable(&self) -> bool {
        !self.launch_executable.is_empty() || self.is_managed
    }

    fn into_game(self) -> EpicGame {
        EpicGame {
            has_expected_dlc: self.expected_dlc.is_some_and(|dlc| !dlc.is_empty()),
            display_name: self.display_name,
            install_location: self.install_location,
            launch_executable: self.launch_executable,
            app_name: self.app_name,
            catalog_namespace: self.catalog_namespace,
            catalog_item_id: self.catalog_item_id,
            is_managed: self.is_managed,
            requires_auth: self.requires_auth,
        }
    }
}

pub(super) fn scan(custom_path: Option<&Path>) -> AppResult<Option<EpicLibrary>> {
    let Some(paths) = find_paths(custom_path) else {
        return Ok(None);
    };
    tracing::debug!(
        launcher = %paths.launcher.path.display(),
        manifests = %paths.manifest_folder_path.display(),
        "Epic Games Launcher found"
    );

    let mut games = BTreeMap::<String, EpicGame>::new();
    for entry in fs::read_dir(&paths.manifest_folder_path)
        .map_err(io_context(&paths.manifest_folder_path))?
    {
        let entry = entry.map_err(io_context(&paths.manifest_folder_path))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let Some(manifest) = read_launcher_json::<Manifest>(&path) else {
            continue;
        };

        // On Linux via Proton, translate Windows paths to host paths via dosdevices
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut manifest = manifest;
        #[cfg(all(unix, not(target_os = "macos")))]
        if let Some(ref compat) = paths.launcher.compat_folder {
            if let Some(translated) =
                crate::importers::wine::translate_windows_path(compat, &manifest.manifest_location)
            {
                manifest.manifest_location = translated.to_string_lossy().to_string();
            }
            if let Some(translated) =
                crate::importers::wine::translate_windows_path(compat, &manifest.install_location)
            {
                manifest.install_location = translated.to_string_lossy().to_string();
            }
        }

        if !manifest.is_installed() {
            tracing::debug!(
                game = manifest.display_name,
                location = manifest.manifest_location,
                "Skipping Epic game that is no longer installed"
            );
            continue;
        }
        if !manifest.is_launchable() {
            tracing::debug!(
                game = manifest.display_name,
                "Skipping Epic item without an executable"
            );
            continue;
        }

        let game = manifest.into_game();
        games.insert(game.dedupe_key(), game);
    }

    Ok(Some(EpicLibrary {
        launcher: paths.launcher,
        games: games.into_values().collect(),
    }))
}

fn find_paths(custom_path: Option<&Path>) -> Option<LegacyPaths> {
    #[cfg(windows)]
    {
        let manifest_folder_path = custom_path.map(PathBuf::from).unwrap_or_else(|| {
            manifest_location_from_registry().unwrap_or_else(default_manifest_location)
        });
        let launcher_path =
            launcher_location_from_registry().unwrap_or_else(default_launcher_location);
        existing_paths(launcher_path, manifest_folder_path)
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        let manifest_folder_path = custom_path
            .map(PathBuf::from)
            .unwrap_or_else(|| macos_manifest_location(&home));
        existing_paths(macos_launcher_location(), manifest_folder_path)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let compat_dir = crate::importers::wine::compat_data_dir()?;

        for entry in std::fs::read_dir(&compat_dir)
            .inspect_err(|error| {
                tracing::warn!(path = %compat_dir.display(), %error, "Could not list Proton prefixes");
            })
            .ok()?
            .flatten()
        {
            let binaries = entry
                .path()
                .join("pfx")
                .join("drive_c")
                .join("Program Files (x86)")
                .join("Epic Games")
                .join("Launcher")
                .join("Portal")
                .join("Binaries");

            let Some(launcher_path) = ["Win64", "Win32"]
                .iter()
                .map(|arch| binaries.join(arch).join("EpicGamesLauncher.exe"))
                .find(|p| p.exists())
            else {
                continue;
            };

            let manifest_folder_path = custom_path.map(PathBuf::from).unwrap_or_else(|| {
                entry
                    .path()
                    .join("pfx")
                    .join("drive_c")
                    .join("ProgramData")
                    .join("Epic")
                    .join("EpicGamesLauncher")
                    .join("Data")
                    .join("Manifests")
            });

            if !manifest_folder_path.exists() {
                tracing::debug!(
                    path = %manifest_folder_path.display(),
                    "Epic Games Launcher found in a Proton prefix, but without manifests"
                );
                continue;
            }
            return Some(LegacyPaths {
                launcher: EpicLauncher {
                    path: launcher_path,
                    compat_folder: Some(entry.path()),
                },
                manifest_folder_path,
            });
        }
        None
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn existing_paths(launcher_path: PathBuf, manifest_folder_path: PathBuf) -> Option<LegacyPaths> {
    for path in [&launcher_path, &manifest_folder_path] {
        if !path.exists() {
            tracing::debug!(path = %path.display(), "Epic Games Launcher path not found");
            return None;
        }
    }
    Some(LegacyPaths {
        launcher: EpicLauncher {
            path: launcher_path,
            compat_folder: None,
        },
        manifest_folder_path,
    })
}

#[cfg(windows)]
fn manifest_location_from_registry() -> Option<PathBuf> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher")
        .ok()?;
    let app_data_path: String = key.get_value("AppDataPath").ok()?;
    let path = Path::new(&app_data_path).join("Manifests");
    path.exists().then_some(path)
}

#[cfg(windows)]
fn launcher_location_from_registry() -> Option<PathBuf> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Classes\\com.epicgames.launcher\\shell\\open\\command")
        .ok()?;
    let command: String = key.get_value("").ok()?;
    parse_quoted_executable(&command).filter(|p| p.exists())
}

#[cfg(windows)]
fn default_launcher_location() -> PathBuf {
    let system_drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
    Path::new(&format!("{system_drive}\\"))
        .join("Program Files (x86)")
        .join("Epic Games")
        .join("Launcher")
        .join("Portal")
        .join("Binaries")
        .join("Win64")
        .join("EpicGamesLauncher.exe")
}

#[cfg(windows)]
fn default_manifest_location() -> PathBuf {
    let program_data =
        std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    Path::new(&program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests")
}

#[cfg(target_os = "macos")]
fn macos_manifest_location(home: &str) -> PathBuf {
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests")
}

#[cfg(target_os = "macos")]
fn macos_launcher_location() -> PathBuf {
    PathBuf::from("/Applications")
        .join("Epic Games Launcher.app")
        .join("Contents")
        .join("MacOS")
        .join("EpicGamesLauncher")
}

#[cfg(windows)]
fn parse_quoted_executable(command: &str) -> Option<PathBuf> {
    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }
    command
        .split_whitespace()
        .next()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> Manifest {
        Manifest {
            launch_executable: "Binaries/Win64/game.exe".to_string(),
            manifest_location: "/manifests/game.item".to_string(),
            display_name: "Test Game".to_string(),
            install_location: "/games/test".to_string(),
            app_name: "testgame123".to_string(),
            catalog_namespace: "ns123".to_string(),
            catalog_item_id: "item456".to_string(),
            is_managed: false,
            requires_auth: false,
            expected_dlc: None,
        }
    }

    #[test]
    fn expected_dlc_prefers_the_launcher() {
        let mut m = test_manifest();
        m.expected_dlc = Some(HashMap::from([("dlc1".to_string(), true)]));
        let game = m.into_game();
        assert!(game.prefers_launcher());
        assert!(!game.requires_launcher());
    }

    #[test]
    fn no_launcher_for_empty_dlc_map() {
        let mut m = test_manifest();
        m.expected_dlc = Some(HashMap::new());
        let game = m.into_game();
        assert!(!game.requires_launcher());
        assert!(!game.prefers_launcher());
    }

    #[test]
    fn manifest_auth_requirement_selects_launcher() {
        for requires_auth in [Some(true), Some(false), None] {
            let mut value = serde_json::json!({
                "LaunchExecutable": "Binaries/Win64/game.exe",
                "ManifestLocation": "/manifests/game.item",
                "DisplayName": "Test Game",
                "InstallLocation": "/games/test",
                "AppName": "testgame123",
                "CatalogNamespace": "ns123",
                "CatalogItemId": "item456",
                "bIsManaged": false,
                "ExpectingDLCInstalled": {},
                "bCanRunOffline": true
            });
            if let Some(requires_auth) = requires_auth {
                value["bRequiresAuth"] = requires_auth.into();
            }

            let manifest: Manifest = serde_json::from_value(value).unwrap();
            assert_eq!(
                manifest.into_game().prefers_launcher(),
                requires_auth.unwrap_or(false),
                "bRequiresAuth = {requires_auth:?}"
            );
        }
    }

    #[test]
    fn launchable_when_managed_without_an_executable() {
        let mut m = test_manifest();
        m.launch_executable = String::new();
        assert!(!m.is_launchable());
        m.is_managed = true;
        assert!(m.is_launchable());
    }

    #[cfg(windows)]
    #[test]
    fn parse_quoted_exe_with_quotes() {
        let cmd = r#""C:\Program Files (x86)\Epic Games\launcher.exe" --flag"#;
        assert_eq!(
            parse_quoted_executable(cmd),
            Some(PathBuf::from(
                r"C:\Program Files (x86)\Epic Games\launcher.exe"
            ))
        );
    }

    #[cfg(windows)]
    #[test]
    fn parse_quoted_exe_without_quotes() {
        let cmd = r"C:\Games\launcher.exe --flag";
        assert_eq!(
            parse_quoted_executable(cmd),
            Some(PathBuf::from(r"C:\Games\launcher.exe"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn parse_quoted_exe_empty_returns_none() {
        assert_eq!(parse_quoted_executable(""), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_manifest_path_uses_library_support() {
        assert_eq!(
            macos_manifest_location("/Users/test"),
            PathBuf::from(
                "/Users/test/Library/Application Support/Epic/EpicGamesLauncher/Data/Manifests"
            )
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_launcher_path_points_to_applications() {
        assert_eq!(
            macos_launcher_location(),
            PathBuf::from("/Applications/Epic Games Launcher.app/Contents/MacOS/EpicGamesLauncher")
        );
    }
}
