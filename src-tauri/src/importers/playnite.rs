use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
    util::litedb,
};
use bson::{spec::BinarySubtype, Bson, Document};
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let (launcher_path, db_path) = find_paths(custom_path)?;
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let bytes = std::fs::read(&db_path).map_err(|e| {
        if e.raw_os_error() == Some(32) {
            AppError::Message(
                "Playnite appears to be running and has locked its database. Close Playnite and try again.".to_string(),
            )
        } else {
            AppError::Io { path: db_path.clone(), source: e }
        }
    })?;

    let games = litedb::read_documents(&bytes).map_err(|error| {
        AppError::Message(format!(
            "Could not read Playnite database at {}: {error}",
            db_path.display()
        ))
    })?;

    let candidates = games
        .iter()
        .filter_map(GameEntry::from_document)
        .filter(|g| g.installed)
        .map(|game| {
            launcher_candidate(
                user,
                ImportSource::Playnite,
                "playnite",
                game.name,
                launcher_path.clone(),
                format!("--hidesplashscreen --start {}", game.id),
                vec!["Playnite".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

fn find_paths(custom_path: Option<&Path>) -> AppResult<(PathBuf, PathBuf)> {
    let install_dir = if let Some(custom) = custom_path {
        custom.to_path_buf()
    } else {
        let local = std::env::var("LOCALAPPDATA")
            .map_err(|_| AppError::Message("LOCALAPPDATA not set".to_string()))?;
        Path::new(&local).join("Playnite")
    };
    let launcher = install_dir.join("Playnite.DesktopApp.exe");
    if !launcher.exists() {
        return Err(AppError::Message("Playnite is not installed".to_string()));
    }
    let appdata =
        std::env::var("APPDATA").map_err(|_| AppError::Message("APPDATA not set".to_string()))?;

    // Playnite treats installs without an uninstaller as portable and keeps data beside the exe
    let config_root = if install_dir.join("unins000.exe").exists() {
        Path::new(&appdata).join("Playnite")
    } else {
        install_dir.clone()
    };
    let db_dir = std::fs::read_to_string(config_root.join("config.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<PlayniteConfig>(&raw).ok())
        .and_then(|c| c.database_path)
        .filter(|p| !p.trim().is_empty())
        .map(|p| expand_db_path(&p, &install_dir, &appdata))
        .unwrap_or_else(|| config_root.join("library"));
    Ok((launcher, db_dir.join("games.db")))
}

#[derive(serde::Deserialize)]
struct PlayniteConfig {
    #[serde(rename = "DatabasePath")]
    database_path: Option<String>,
}

/// Mirrors Playnite's `GameDatabase.GetFullDbPath`.
fn expand_db_path(path: &str, install_dir: &Path, appdata: &str) -> PathBuf {
    const PLAYNITE_DIR: &str = "{playnitedir}";
    const APPDATA: &str = "%appdata%";
    let lower = path.to_ascii_lowercase();
    let expanded = if let Some(i) = lower.find(PLAYNITE_DIR) {
        format!(
            "{}{}{}",
            &path[..i],
            install_dir.display(),
            &path[i + PLAYNITE_DIR.len()..]
        )
    } else if let Some(i) = lower.find(APPDATA) {
        format!("{}{}{}", &path[..i], appdata, &path[i + APPDATA.len()..])
    } else {
        path.to_string()
    };
    let expanded = PathBuf::from(expanded);
    if expanded.is_absolute() {
        expanded
    } else {
        install_dir.join(expanded)
    }
}

struct GameEntry {
    name: String,
    id: String,
    installed: bool,
}

impl GameEntry {
    fn from_document(doc: &Document) -> Option<Self> {
        let id = match doc.get("_id")? {
            // LiteDB stores .NET's `Guid.ToByteArray`, whose first three groups are little-endian
            Bson::Binary(binary) if binary.subtype == BinarySubtype::Uuid => {
                uuid::Uuid::from_slice_le(&binary.bytes).ok()?.to_string()
            }
            Bson::String(id) => id.clone(),
            _ => return None,
        };
        Some(Self {
            name: doc.get_str("Name").ok()?.to_string(),
            id,
            installed: doc.get_bool("IsInstalled").unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guid_doc(bytes: [u8; 16]) -> Document {
        bson::doc! {
            "_id": bson::Binary { subtype: BinarySubtype::Uuid, bytes: bytes.to_vec() },
            "Name": "The Witcher 3",
            "IsInstalled": true,
        }
    }

    #[test]
    fn formats_ids_like_dotnet() {
        let doc = guid_doc([
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        let game = GameEntry::from_document(&doc).unwrap();
        assert_eq!(game.id, "00112233-4455-6677-8899-aabbccddeeff");
        assert_eq!(game.name, "The Witcher 3");
        assert!(game.installed);
    }

    #[test]
    fn accepts_string_ids_and_defaults_to_not_installed() {
        let doc = bson::doc! { "_id": "abc", "Name": "Hades" };
        let game = GameEntry::from_document(&doc).unwrap();
        assert_eq!(game.id, "abc");
        assert!(!game.installed);
    }

    #[test]
    fn rejects_documents_that_are_not_games() {
        assert!(GameEntry::from_document(&bson::doc! { "_id": "abc" }).is_none());
        assert!(GameEntry::from_document(&bson::doc! { "Name": "No id" }).is_none());
        let mut short_guid = guid_doc([0; 16]);
        short_guid.insert(
            "_id",
            bson::Binary {
                subtype: BinarySubtype::Uuid,
                bytes: vec![1, 2],
            },
        );
        assert!(GameEntry::from_document(&short_guid).is_none());
    }

    #[test]
    fn expands_playnite_dir_variable() {
        let path = expand_db_path(r"{PlayniteDir}\library", Path::new(r"D:\Playnite"), "");
        assert_eq!(path, PathBuf::from(r"D:\Playnite\library"));
    }

    #[test]
    fn expands_appdata_case_insensitively() {
        let path = expand_db_path(
            r"%APPDATA%\Playnite\library",
            Path::new(r"D:\Playnite"),
            r"C:\Users\me\AppData\Roaming",
        );
        assert_eq!(
            path,
            PathBuf::from(r"C:\Users\me\AppData\Roaming\Playnite\library")
        );
    }

    #[test]
    fn keeps_absolute_paths_and_resolves_relative_ones() {
        let install = Path::new(r"D:\Playnite");
        assert_eq!(
            expand_db_path(r"E:\PlayniteDb", install, ""),
            PathBuf::from(r"E:\PlayniteDb")
        );
        assert_eq!(
            expand_db_path("library", install, ""),
            PathBuf::from(r"D:\Playnite\library")
        );
    }
}
