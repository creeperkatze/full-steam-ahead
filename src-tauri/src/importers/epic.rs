use crate::{
    error::{io_context, AppResult},
    importers::{candidate_from_parts, launcher_candidate, launcher_url_pair, read_launcher_json},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
struct EpicPaths {
    launcher_path: PathBuf,
    manifest_folder_path: PathBuf,
    #[cfg_attr(not(unix), allow(dead_code))]
    compat_folder: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
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

impl EpicManifest {
    fn launch_url(&self) -> String {
        format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            self.catalog_namespace, self.catalog_item_id, self.app_name
        )
    }

    fn dedupe_key(&self) -> String {
        format!(
            "{}-{}-{}",
            self.install_location, self.launch_executable, self.is_managed
        )
    }

    fn has_expected_dlc(&self) -> bool {
        self.expected_dlc
            .as_ref()
            .is_some_and(|dlc| !dlc.is_empty())
    }

    /// Games with no executable of their own to start.
    fn requires_launcher(&self) -> bool {
        self.is_managed || self.launch_executable.is_empty()
    }

    /// Adds titles the launcher should handle by default: DLC it assembles, and
    /// titles wanting auth arguments that only it passes.
    fn prefers_launcher(&self) -> bool {
        self.requires_launcher() || self.requires_auth || self.has_expected_dlc()
    }

    fn executable_path(&self) -> PathBuf {
        Path::new(&self.install_location).join(&self.launch_executable)
    }
}

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let Some(paths) = find_epic_paths(custom_path) else {
        return Ok(Vec::new());
    };
    tracing::debug!(
        launcher = %paths.launcher_path.display(),
        manifests = %paths.manifest_folder_path.display(),
        "Epic Games Launcher found"
    );

    let mut manifests = BTreeMap::<String, EpicManifest>::new();
    for entry in fs::read_dir(&paths.manifest_folder_path)
        .map_err(io_context(&paths.manifest_folder_path))?
    {
        let entry = entry.map_err(io_context(&paths.manifest_folder_path))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let Some(manifest) = read_launcher_json::<EpicManifest>(&path) else {
            continue;
        };

        // On Linux via Proton, translate Windows paths to host paths via dosdevices
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut manifest = manifest;
        #[cfg(all(unix, not(target_os = "macos")))]
        if let Some(ref compat) = paths.compat_folder {
            if let Some(translated) =
                super::translate_windows_path(compat, &manifest.manifest_location)
            {
                manifest.manifest_location = translated.to_string_lossy().to_string();
            }
            if let Some(translated) =
                super::translate_windows_path(compat, &manifest.install_location)
            {
                manifest.install_location = translated.to_string_lossy().to_string();
            }
        }

        if !is_installed(&manifest) {
            tracing::debug!(
                game = manifest.display_name,
                location = manifest.manifest_location,
                "Skipping Epic game that is no longer installed"
            );
            continue;
        }
        if !is_launchable(&manifest) {
            tracing::debug!(
                game = manifest.display_name,
                "Skipping Epic item without an executable"
            );
            continue;
        }

        manifests.insert(manifest.dedupe_key(), manifest);
    }

    Ok(manifests
        .into_values()
        .map(|manifest| candidate_from_manifest(user, &paths, manifest))
        .collect())
}

fn candidate_from_manifest(
    user: &SteamUser,
    paths: &EpicPaths,
    manifest: EpicManifest,
) -> ImportCandidate {
    let requires_launcher = manifest.requires_launcher();
    let prefers_launcher = manifest.prefers_launcher();
    let name = manifest.display_name.clone();
    let launch_url = manifest.launch_url();
    let exe = manifest.executable_path();
    let mut tags = vec!["Epic".to_string()];
    if prefers_launcher {
        tags.push("Epic Launcher".to_string());
    }

    // On Linux with Proton, embed the compat path into the launch options
    #[cfg(all(unix, not(target_os = "macos")))]
    let launch_url = if let Some(ref compat) = paths.compat_folder {
        format!(
            "STEAM_COMPAT_DATA_PATH=\"{}\" %command% -'{}'",
            compat.display(),
            launch_url
        )
    } else {
        launch_url
    };

    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(unused_mut))]
    let mut candidate = if requires_launcher {
        launcher_candidate(
            user,
            ImportSource::Epic,
            "epic",
            name,
            paths.launcher_path.clone(),
            launch_url,
            tags,
        )
    } else {
        // Keeps both routes so the user can switch; this only picks the default.
        let start_dir = exe.parent().map(PathBuf::from).unwrap_or_default();
        let mut candidate = candidate_from_parts(
            user,
            ImportSource::Epic,
            "epic",
            name,
            exe,
            start_dir,
            None,
            tags,
        );
        let (launcher_path, launch_url) =
            launcher_url_pair(paths.launcher_path.clone(), launch_url);
        candidate.url_scheme = Some(launch_url);
        candidate.launcher_path = Some(launcher_path);
        candidate.use_launcher_url = prefers_launcher;
        candidate
    };

    // Found inside an existing Proton prefix, so Steam must be told to launch it via Proton
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        candidate.needs_proton = paths.compat_folder.is_some();
    }

    candidate
}

fn is_installed(manifest: &EpicManifest) -> bool {
    Path::new(&manifest.manifest_location).exists()
}

fn is_launchable(manifest: &EpicManifest) -> bool {
    !manifest.launch_executable.is_empty() || manifest.is_managed
}

fn find_epic_paths(custom_path: Option<&Path>) -> Option<EpicPaths> {
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
        let compat_dir = super::compat_data_dir()?;

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
            return Some(EpicPaths {
                launcher_path,
                manifest_folder_path,
                compat_folder: Some(entry.path()),
            });
        }
        None
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn existing_paths(launcher_path: PathBuf, manifest_folder_path: PathBuf) -> Option<EpicPaths> {
    for path in [&launcher_path, &manifest_folder_path] {
        if !path.exists() {
            tracing::debug!(path = %path.display(), "Epic Games Launcher path not found");
            return None;
        }
    }
    Some(EpicPaths {
        launcher_path,
        manifest_folder_path,
        compat_folder: None,
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
    use std::collections::HashMap;

    fn test_manifest() -> EpicManifest {
        EpicManifest {
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
    fn needs_launcher_when_managed() {
        let mut m = test_manifest();
        m.is_managed = true;
        assert!(m.requires_launcher());
        assert!(m.prefers_launcher());
    }

    #[test]
    fn needs_launcher_when_has_dlc() {
        let mut m = test_manifest();
        m.expected_dlc = Some(HashMap::from([("dlc1".to_string(), true)]));
        assert!(m.prefers_launcher());
        // Has an executable of its own, so the user can still switch to it.
        assert!(!m.requires_launcher());
    }

    #[test]
    fn requires_launcher_without_an_executable() {
        let mut m = test_manifest();
        m.launch_executable = String::new();
        assert!(m.requires_launcher());
    }

    #[test]
    fn no_launcher_for_plain_game() {
        assert!(!test_manifest().requires_launcher());
        assert!(!test_manifest().prefers_launcher());
    }

    #[test]
    fn no_launcher_for_empty_dlc_map() {
        let mut m = test_manifest();
        m.expected_dlc = Some(HashMap::new());
        assert!(!m.requires_launcher());
        assert!(!m.prefers_launcher());
    }

    #[test]
    fn auth_prefers_the_launcher_but_keeps_the_executable() {
        let mut m = test_manifest();
        m.requires_auth = true;
        assert!(m.prefers_launcher());
        // Still has an executable of its own, so the user can switch back to it.
        assert!(!m.requires_launcher());
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

            let manifest: EpicManifest = serde_json::from_value(value).unwrap();
            assert_eq!(
                manifest.prefers_launcher(),
                requires_auth.unwrap_or(false),
                "bRequiresAuth = {requires_auth:?}"
            );
        }
    }

    #[test]
    fn launch_url_format() {
        let m = test_manifest();
        assert_eq!(
            m.launch_url(),
            "com.epicgames.launcher://apps/ns123%3Aitem456%3Atestgame123?action=launch&silent=true"
        );
    }

    #[test]
    fn dedupe_key_includes_all_parts() {
        let m = test_manifest();
        assert_eq!(m.dedupe_key(), "/games/test-Binaries/Win64/game.exe-false");
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

    #[test]
    fn executable_path_joins_location_and_exe() {
        let m = test_manifest();
        assert_eq!(
            m.executable_path(),
            PathBuf::from("/games/test/Binaries/Win64/game.exe")
        );
    }
}
