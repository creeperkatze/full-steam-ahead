//! The rewritten Epic Games Launcher keeps its library per user in `%LOCALAPPDATA%\com.epicgames.eglv2`.

use super::{EpicGame, EpicLauncher, EpicLibrary};
use crate::importers::{parse_launcher_json, read_launcher_file, read_launcher_json};
use serde::{de::IgnoredAny, Deserialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

const INSTALLATIONS_FILE: &str = "installations.jsonl";
const INSTALLATIONS_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct InstallationsHeader {
    version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallationRecord {
    installation_id: String,
    #[serde(default)]
    deleted: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Installation {
    #[serde(default)]
    complete: bool,
    dir: String,
    namespace: String,
    item_id: String,
    artifact_id: String,
    #[serde(default)]
    launch_executable: String,
    parent_installation_id: Option<String>,
    #[serde(default)]
    is_fab: bool,
}

impl Installation {
    fn is_addon(&self) -> bool {
        self.parent_installation_id
            .as_ref()
            .is_some_and(|id| !id.is_empty())
    }

    fn folder_name(&self) -> String {
        Path::new(&self.dir)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.dir.clone())
    }
}

#[derive(Debug, Deserialize)]
struct CatalogCache {
    items: HashMap<String, CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    item: CatalogItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogItem {
    title: String,
    main_game_item: Option<IgnoredAny>,
}

pub(super) fn is_data_folder(path: &Path) -> bool {
    path.join(INSTALLATIONS_FILE).exists()
}

pub(super) fn scan(custom_path: Option<&Path>) -> Option<EpicLibrary> {
    let local_data = dirs::data_local_dir()?;
    let data_path = custom_path
        .map(PathBuf::from)
        .unwrap_or_else(|| local_data.join("com.epicgames.eglv2"));
    let launcher_path = local_data
        .join("Epic Games")
        .join("Launcher")
        .join("EpicGamesLauncher.exe");
    let installations_path = data_path.join(INSTALLATIONS_FILE);
    for path in [&launcher_path, &installations_path] {
        if !path.exists() {
            tracing::debug!(path = %path.display(), "New Epic Games Launcher path not found");
            return None;
        }
    }
    tracing::debug!(
        launcher = %launcher_path.display(),
        data = %data_path.display(),
        "New Epic Games Launcher found"
    );

    let raw = read_launcher_file(&installations_path)?;
    let catalog_path = data_path
        .join("cache")
        .join("prod")
        .join("catalog-cache.json");
    let catalog = read_launcher_json::<CatalogCache>(&catalog_path)
        .map(|cache| cache.items)
        .unwrap_or_default();

    let games = parse_installations(&installations_path.display().to_string(), &raw)
        .into_iter()
        .filter_map(|installation| game_from_installation(installation, &catalog))
        .collect();
    Some(EpicLibrary {
        launcher: EpicLauncher {
            path: launcher_path,
            compat_folder: None,
        },
        games,
    })
}

/// Each line replaces the earlier record with the same id, and a deleted record removes it.
fn parse_installations(origin: &str, raw: &str) -> Vec<Installation> {
    let mut lines = raw.lines().filter(|line| !line.trim().is_empty());
    if let Some(header) = lines
        .next()
        .and_then(|line| parse_launcher_json::<InstallationsHeader>(origin, line))
    {
        if header.version != INSTALLATIONS_VERSION {
            tracing::warn!(
                origin,
                version = header.version,
                "Unknown version of the Epic installations file"
            );
        }
    }

    let mut installations = BTreeMap::new();
    for line in lines {
        let Some(record) = parse_launcher_json::<InstallationRecord>(origin, line) else {
            continue;
        };
        if record.deleted {
            installations.remove(&record.installation_id);
        } else if let Some(installation) = parse_launcher_json::<Installation>(origin, line) {
            installations.insert(record.installation_id, installation);
        }
    }
    installations.into_values().collect()
}

fn game_from_installation(
    installation: Installation,
    catalog: &HashMap<String, CatalogEntry>,
) -> Option<EpicGame> {
    let item = catalog.get(&installation.item_id).map(|entry| &entry.item);
    let name = match item {
        Some(item) => item.title.clone(),
        None => {
            tracing::debug!(
                item = installation.item_id,
                dir = installation.dir,
                "Epic game missing from the catalog cache, using its folder name"
            );
            installation.folder_name()
        }
    };

    if !installation.complete {
        tracing::debug!(game = name, "Skipping Epic game that isn't fully installed");
        return None;
    }
    if installation.is_fab {
        tracing::debug!(game = name, "Skipping Fab asset");
        return None;
    }
    if installation.is_addon() || item.is_some_and(|item| item.main_game_item.is_some()) {
        tracing::debug!(game = name, "Skipping Epic DLC");
        return None;
    }
    if installation.launch_executable.is_empty() {
        tracing::debug!(game = name, "Skipping Epic item without an executable");
        return None;
    }

    Some(EpicGame {
        display_name: name,
        install_location: installation.dir,
        launch_executable: installation.launch_executable,
        app_name: installation.artifact_id,
        catalog_namespace: installation.namespace,
        catalog_item_id: installation.item_id,
        is_managed: false,
        // The new launcher doesn't record whether a game needs its auth arguments.
        requires_auth: true,
        has_expected_dlc: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = r#"{"version":1,"writtenAt":"2026-10-20T16:56:06Z","entries":1}"#;
    const DOOM: &str = r#"{"installationId":"6ED92F850D4F7125AB6CE4CFF7107925","complete":true,"dir":"C:\\Program Files\\Epic Games\\DOOM64","platform":"Windows","namespace":"0ea70edafcc54b3191e262bbb6971981","itemId":"886507fcf6b94d98a02a018418fffd1f","artifactId":"5b60142e120c4f2d88027595c21d4a04","version":"2124","tags":[""],"sidecar":{"revision":0},"launchCommand":"","launchExecutable":"DOOM64_x64.exe","writtenAt":"2026-10-20T16:55:25Z"}"#;

    fn catalog(entries: &[(&str, &str, bool)]) -> HashMap<String, CatalogEntry> {
        let items = entries
            .iter()
            .map(|(id, title, dlc)| {
                let main_game = if *dlc {
                    serde_json::json!({ "id": "main" })
                } else {
                    serde_json::Value::Null
                };
                (
                    id.to_string(),
                    serde_json::json!({ "item": { "id": id, "title": title, "mainGameItem": main_game } }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let cache: CatalogCache = serde_json::from_value(serde_json::json!({
            "items": items,
            "schemaVersion": 1,
            "locale": "de"
        }))
        .unwrap();
        cache.items
    }

    fn games(raw: &str, catalog: &HashMap<String, CatalogEntry>) -> Vec<EpicGame> {
        parse_installations("test", raw)
            .into_iter()
            .filter_map(|installation| game_from_installation(installation, catalog))
            .collect()
    }

    #[test]
    fn reads_an_installation_written_by_the_launcher() {
        let catalog = catalog(&[("886507fcf6b94d98a02a018418fffd1f", "DOOM 64", false)]);
        let found = games(&format!("{HEADER}\n{DOOM}\n"), &catalog);

        assert_eq!(found.len(), 1);
        let doom = &found[0];
        assert_eq!(doom.display_name, "DOOM 64");
        assert_eq!(
            doom.executable_path(),
            PathBuf::from(r"C:\Program Files\Epic Games\DOOM64\DOOM64_x64.exe")
        );
        assert_eq!(
            doom.launch_url(),
            "com.epicgames.launcher://apps/0ea70edafcc54b3191e262bbb6971981%3A886507fcf6b94d98a02a018418fffd1f%3A5b60142e120c4f2d88027595c21d4a04?action=launch&silent=true"
        );
        assert!(doom.prefers_launcher());
        assert!(!doom.requires_launcher());
    }

    #[test]
    fn falls_back_to_the_folder_name_without_a_catalog_entry() {
        let found = games(&format!("{HEADER}\n{DOOM}"), &HashMap::new());
        assert_eq!(found[0].display_name, "DOOM64");
    }

    #[test]
    fn later_records_replace_earlier_ones() {
        let moved = DOOM.replace(r"Epic Games\\DOOM64", r"Games\\DOOM64");
        let found = games(&format!("{HEADER}\n{DOOM}\n{moved}"), &HashMap::new());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].install_location, r"C:\Program Files\Games\DOOM64");
    }

    #[test]
    fn deleted_records_remove_the_installation() {
        let tombstone = r#"{"installationId":"6ED92F850D4F7125AB6CE4CFF7107925","deleted":true,"writtenAt":"2026-10-21T10:00:00Z"}"#;
        let found = games(&format!("{HEADER}\n{DOOM}\n{tombstone}"), &HashMap::new());
        assert!(found.is_empty());
    }

    #[test]
    fn skips_incomplete_installs() {
        let incomplete = DOOM.replace(r#""complete":true"#, r#""complete":false"#);
        assert!(games(&format!("{HEADER}\n{incomplete}"), &HashMap::new()).is_empty());
    }

    #[test]
    fn skips_dlc() {
        let addon = DOOM.replace(
            r#""complete":true"#,
            r#""complete":true,"parentInstallationId":"ABC""#,
        );
        assert!(games(&format!("{HEADER}\n{addon}"), &HashMap::new()).is_empty());

        let catalog = catalog(&[("886507fcf6b94d98a02a018418fffd1f", "Expansion", true)]);
        assert!(games(&format!("{HEADER}\n{DOOM}"), &catalog).is_empty());
    }

    #[test]
    fn skips_fab_assets() {
        let fab = DOOM.replace(r#""complete":true"#, r#""complete":true,"isFab":true"#);
        assert!(games(&format!("{HEADER}\n{fab}"), &HashMap::new()).is_empty());
    }

    #[test]
    fn skips_items_without_an_executable() {
        let tool = DOOM.replace(
            r#""launchExecutable":"DOOM64_x64.exe""#,
            r#""launchExecutable":"""#,
        );
        assert!(games(&format!("{HEADER}\n{tool}"), &HashMap::new()).is_empty());
    }

    #[test]
    fn skips_a_torn_last_line() {
        let torn = &DOOM[..DOOM.len() / 2];
        let found = games(&format!("{HEADER}\n{DOOM}\n{torn}"), &HashMap::new());
        assert_eq!(found.len(), 1);
    }
}
