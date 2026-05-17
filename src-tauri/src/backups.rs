use crate::{
    error::{io_context, AppError, AppResult},
    models::{BackupInfo, BackupPlan, SteamUser},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, info, warn};

const MANIFEST_FILENAME: &str = "manifest.json";

#[derive(Debug, Serialize, Deserialize)]
struct BackupManifest {
    files: HashMap<String, PathBuf>,
}

pub fn list() -> AppResult<Vec<BackupInfo>> {
    list_from_dir(&crate::paths::app_data_dir().join("backups"))
}

pub fn restore(backup_id: &str, user: &SteamUser) -> AppResult<usize> {
    let backup_dir = crate::paths::app_data_dir().join("backups").join(backup_id);
    restore_from_dir(&backup_dir, user)
}

pub fn write_manifest(backup_dir: &Path, plans: &[BackupPlan]) {
    let files: HashMap<String, PathBuf> = plans
        .iter()
        .filter(|b| b.destination.exists())
        .filter_map(|b| {
            b.destination
                .file_name()
                .and_then(|n| n.to_str())
                .map(|name| (name.to_string(), b.source.clone()))
        })
        .collect();

    if files.is_empty() {
        return;
    }

    let manifest = BackupManifest { files };
    match serde_json::to_string_pretty(&manifest) {
        Ok(json) => {
            let path = backup_dir.join(MANIFEST_FILENAME);
            if let Err(e) = fs::write(&path, json) {
                warn!(error = %e, "Failed to write backup manifest");
            }
        }
        Err(e) => warn!(error = %e, "Failed to serialize backup manifest"),
    }
}

fn list_from_dir(backups_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(backups_dir).map_err(io_context(backups_dir))? {
        let entry = entry.map_err(io_context(backups_dir))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let created_at = id_to_iso(&id);
        let mut file_count = 0usize;
        let mut size_bytes = 0u64;
        if let Ok(entries) = fs::read_dir(&path) {
            for file_entry in entries.flatten() {
                let file_path = file_entry.path();
                if !file_path.is_file() {
                    continue;
                }
                // Exclude the manifest from the user-visible file count
                if file_entry.file_name() == MANIFEST_FILENAME {
                    continue;
                }
                file_count += 1;
                if let Ok(meta) = file_entry.metadata() {
                    size_bytes += meta.len();
                }
            }
        }

        backups.push(BackupInfo {
            id,
            created_at,
            file_count,
            size_bytes,
        });
    }

    backups.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(backups)
}

fn restore_from_dir(backup_dir: &Path, user: &SteamUser) -> AppResult<usize> {
    if !backup_dir.exists() {
        return Err(AppError::Message(format!(
            "Backup '{}' not found.",
            backup_dir.display()
        )));
    }

    let manifest = load_manifest(backup_dir);
    if manifest.is_none() {
        debug!("No manifest found, falling back to filename inference");
    }

    let mut restored = 0usize;
    for entry in fs::read_dir(backup_dir).map_err(io_context(backup_dir))? {
        let entry = entry.map_err(io_context(backup_dir))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if filename == MANIFEST_FILENAME {
            continue;
        }

        let destination = manifest
            .as_ref()
            .and_then(|m| m.files.get(&filename))
            .cloned()
            .unwrap_or_else(|| infer_destination(user, &filename));

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(io_context(parent))?;
        }
        fs::copy(&path, &destination).map_err(io_context(&destination))?;
        debug!(src = %path.display(), dst = %destination.display(), "File restored from backup");
        restored += 1;
    }

    info!(restored, "Backup restored");
    Ok(restored)
}

fn load_manifest(backup_dir: &Path) -> Option<BackupManifest> {
    let raw = fs::read_to_string(backup_dir.join(MANIFEST_FILENAME)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn infer_destination(user: &SteamUser, filename: &str) -> PathBuf {
    match filename {
        "shortcuts.vdf" => user.shortcuts_path.clone(),
        "cloud-storage-namespace-1.json" => user.collections_path.clone(),
        other => user.grid_path.join(other),
    }
}

fn id_to_iso(id: &str) -> String {
    if id.len() == 15 && id.as_bytes().get(8) == Some(&b'-') {
        format!(
            "{}-{}-{}T{}:{}:{}Z",
            &id[0..4],
            &id[4..6],
            &id[6..8],
            &id[9..11],
            &id[11..13],
            &id[13..15]
        )
    } else {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Creates a unique temp directory that is cleaned up when the returned
    /// path goes out of scope. Uses std only — no extra dependencies needed.
    struct TmpDir(PathBuf);

    impl TmpDir {
        fn new() -> Self {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("fsa_test_{}_{}", std::process::id(), n));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn make_user(base: &Path) -> SteamUser {
        let grid = base.join("grid");
        fs::create_dir_all(&grid).unwrap();
        SteamUser {
            steam_id: "123".to_string(),
            account_name: None,
            shortcuts_path: base.join("shortcuts.vdf"),
            collections_path: base.join("cloud-storage-namespace-1.json"),
            grid_path: grid,
        }
    }

    fn write(path: &Path, content: &[u8]) {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    // id_to_iso

    #[test]
    fn timestamp_valid_format() {
        assert_eq!(id_to_iso("20250516-143022"), "2025-05-16T14:30:22Z");
    }

    #[test]
    fn timestamp_invalid_falls_back_to_raw() {
        assert_eq!(id_to_iso("not-a-timestamp"), "not-a-timestamp");
        assert_eq!(id_to_iso(""), "");
        assert_eq!(id_to_iso("20250516_143022"), "20250516_143022");
    }

    // infer_destination

    #[test]
    fn infer_shortcuts_path() {
        let tmp = TmpDir::new();
        let user = make_user(tmp.path());
        assert_eq!(
            infer_destination(&user, "shortcuts.vdf"),
            user.shortcuts_path
        );
    }

    #[test]
    fn infer_collections_path() {
        let tmp = TmpDir::new();
        let user = make_user(tmp.path());
        assert_eq!(
            infer_destination(&user, "cloud-storage-namespace-1.json"),
            user.collections_path
        );
    }

    #[test]
    fn infer_artwork_goes_to_grid() {
        let tmp = TmpDir::new();
        let user = make_user(tmp.path());
        assert_eq!(
            infer_destination(&user, "12345_header.png"),
            user.grid_path.join("12345_header.png")
        );
    }

    // write_manifest / load_manifest

    #[test]
    fn write_and_load_manifest_roundtrip() {
        let tmp = TmpDir::new();
        let dst_shortcuts = tmp.path().join("shortcuts.vdf");
        let dst_art = tmp.path().join("99_header.png");
        write(&dst_shortcuts, b"vdf");
        write(&dst_art, b"png");

        let src_shortcuts = PathBuf::from("/steam/config/shortcuts.vdf");
        let src_art = PathBuf::from("/steam/config/grid/99_header.png");
        let plans = vec![
            BackupPlan {
                source: src_shortcuts.clone(),
                destination: dst_shortcuts,
            },
            BackupPlan {
                source: src_art.clone(),
                destination: dst_art,
            },
        ];
        write_manifest(tmp.path(), &plans);

        let manifest = load_manifest(tmp.path()).expect("manifest should exist");
        assert_eq!(manifest.files["shortcuts.vdf"], src_shortcuts);
        assert_eq!(manifest.files["99_header.png"], src_art);
    }

    #[test]
    fn write_manifest_skips_missing_destinations() {
        let tmp = TmpDir::new();
        // destination does not exist on disk
        let plans = vec![BackupPlan {
            source: PathBuf::from("/original/shortcuts.vdf"),
            destination: tmp.path().join("shortcuts.vdf"),
        }];
        write_manifest(tmp.path(), &plans);
        assert!(!tmp.path().join(MANIFEST_FILENAME).exists());
    }

    #[test]
    fn load_manifest_returns_none_when_absent() {
        let tmp = TmpDir::new();
        assert!(load_manifest(tmp.path()).is_none());
    }

    // restore_from_dir

    #[test]
    fn restore_uses_manifest_paths() {
        let tmp = TmpDir::new();
        let backup_dir = tmp.path().join("backup");
        let user = make_user(tmp.path());

        write(&backup_dir.join("shortcuts.vdf"), b"backup-data");

        let custom_dest = tmp.path().join("custom").join("shortcuts.vdf");
        let manifest = BackupManifest {
            files: [("shortcuts.vdf".to_string(), custom_dest.clone())]
                .into_iter()
                .collect(),
        };
        write(
            &backup_dir.join(MANIFEST_FILENAME),
            serde_json::to_string(&manifest).unwrap().as_bytes(),
        );

        let count = restore_from_dir(&backup_dir, &user).unwrap();
        assert_eq!(count, 1);
        assert_eq!(fs::read(&custom_dest).unwrap(), b"backup-data");
    }

    #[test]
    fn restore_skips_manifest_file_itself() {
        let tmp = TmpDir::new();
        let backup_dir = tmp.path().join("backup");
        let user = make_user(tmp.path());

        write(&backup_dir.join("shortcuts.vdf"), b"data");
        let manifest = BackupManifest {
            files: [("shortcuts.vdf".to_string(), user.shortcuts_path.clone())]
                .into_iter()
                .collect(),
        };
        write(
            &backup_dir.join(MANIFEST_FILENAME),
            serde_json::to_string(&manifest).unwrap().as_bytes(),
        );

        let count = restore_from_dir(&backup_dir, &user).unwrap();
        assert_eq!(count, 1); // only shortcuts.vdf, not manifest.json
    }

    #[test]
    fn restore_errors_when_backup_dir_missing() {
        let tmp = TmpDir::new();
        let user = make_user(tmp.path());
        assert!(restore_from_dir(&tmp.path().join("nonexistent"), &user).is_err());
    }

    // list_from_dir

    #[test]
    fn list_sorted_newest_first() {
        let tmp = TmpDir::new();
        for id in ["20250101-000000", "20250516-120000", "20240601-080000"] {
            let dir = tmp.path().join(id);
            write(&dir.join("shortcuts.vdf"), b"x");
        }

        let result = list_from_dir(tmp.path()).unwrap();
        assert_eq!(result[0].id, "20250516-120000");
        assert_eq!(result[1].id, "20250101-000000");
        assert_eq!(result[2].id, "20240601-080000");
    }

    #[test]
    fn list_excludes_manifest_from_count_and_size() {
        let tmp = TmpDir::new();
        let backup_dir = tmp.path().join("20250516-120000");
        write(&backup_dir.join("shortcuts.vdf"), b"vdf");
        write(&backup_dir.join("cloud-storage-namespace-1.json"), b"{}");
        write(&backup_dir.join(MANIFEST_FILENAME), b"manifest-content");

        let result = list_from_dir(tmp.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_count, 2);
        // size should not include the manifest
        assert_eq!(result[0].size_bytes, 3 + 2); // "vdf" + "{}"
    }

    #[test]
    fn list_returns_empty_when_dir_absent() {
        let tmp = TmpDir::new();
        let result = list_from_dir(&tmp.path().join("nonexistent")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_formats_timestamp_in_display() {
        let tmp = TmpDir::new();
        write(
            &tmp.path().join("20250516-143022").join("shortcuts.vdf"),
            b"x",
        );

        let result = list_from_dir(tmp.path()).unwrap();
        assert_eq!(result[0].created_at, "2025-05-16T14:30:22Z");
    }
}
