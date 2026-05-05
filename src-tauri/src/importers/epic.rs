use crate::{
    error::{io_context, AppResult},
    importers::quote_path,
    models::{ImportCandidate, ImportSource, SteamUser},
    steam::{artwork, non_steam_app_id},
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

    fn needs_launcher(&self) -> bool {
        if self.is_managed {
            return true;
        }

        self.expected_dlc
            .as_ref()
            .is_some_and(|dlc| !dlc.is_empty())
    }

    fn executable_path(&self) -> PathBuf {
        Path::new(&self.install_location).join(&self.launch_executable)
    }
}

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let Some(paths) = find_epic_paths() else {
        return Ok(Vec::new());
    };

    let mut manifests = BTreeMap::<String, EpicManifest>::new();
    for entry in fs::read_dir(&paths.manifest_folder_path)
        .map_err(io_context(&paths.manifest_folder_path))?
    {
        let entry = entry.map_err(io_context(&paths.manifest_folder_path))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("item") {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
        let Ok(manifest) = serde_json::from_str::<EpicManifest>(&raw) else {
            continue;
        };

        if !is_installed(&manifest) || !is_launchable(&manifest) {
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
    let needs_launcher = manifest.needs_launcher();
    let launch_options = needs_launcher.then(|| manifest.launch_url());
    let executable_path = if needs_launcher {
        paths.launcher_path.clone()
    } else {
        manifest.executable_path()
    };
    let start_dir = executable_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default();
    let app_id = non_steam_app_id(&quote_path(&executable_path), &manifest.display_name);
    let mut tags = vec!["Epic".to_string()];
    if needs_launcher {
        tags.push("Epic Launcher".to_string());
    }
    let (matched_steam_app_id, artwork) =
        artwork::steam_preferred_plan(&user.grid_path, app_id, &manifest.display_name);

    ImportCandidate {
        id: format!("epic-{app_id}"),
        source: ImportSource::Epic,
        name: manifest.display_name,
        executable_path,
        start_dir,
        launch_options,
        existing_app_id: None,
        matched_steam_app_id,
        tags,
        artwork,
    }
}

fn is_installed(manifest: &EpicManifest) -> bool {
    Path::new(&manifest.manifest_location).exists()
}

fn is_launchable(manifest: &EpicManifest) -> bool {
    !manifest.launch_executable.is_empty() || manifest.is_managed
}

fn find_epic_paths() -> Option<EpicPaths> {
    let manifest_folder_path =
        manifest_location_from_registry().unwrap_or_else(default_manifest_location);
    let launcher_path = launcher_location_from_registry().unwrap_or_else(default_launcher_location);

    (manifest_folder_path.exists() && launcher_path.exists()).then_some(EpicPaths {
        launcher_path,
        manifest_folder_path,
    })
}

fn manifest_location_from_registry() -> Option<PathBuf> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher")
        .ok()?;
    let app_data_path: String = key.get_value("AppDataPath").ok()?;
    let path = Path::new(&app_data_path).join("Manifests");
    path.exists().then_some(path)
}

fn launcher_location_from_registry() -> Option<PathBuf> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Classes\\com.epicgames.launcher\\shell\\open\\command")
        .ok()?;
    let command: String = key.get_value("").ok()?;
    parse_quoted_executable(&command).filter(|path| path.exists())
}

fn parse_quoted_executable(command: &str) -> Option<PathBuf> {
    if let Some(rest) = command.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(PathBuf::from(&rest[..end]));
    }

    command
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

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

fn default_manifest_location() -> PathBuf {
    let program_data =
        std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    Path::new(&program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests")
}
