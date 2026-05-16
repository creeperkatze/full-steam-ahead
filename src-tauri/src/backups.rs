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
    /// Maps the backed-up filename to its original source path.
    files: HashMap<String, PathBuf>,
}

pub fn list() -> AppResult<Vec<BackupInfo>> {
    let backups_dir = crate::paths::app_data_dir().join("backups");
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&backups_dir).map_err(io_context(&backups_dir))? {
        let entry = entry.map_err(io_context(&backups_dir))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let created_at = format_timestamp(&id);
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

/// Writes a manifest.json into `backup_dir` mapping each backed-up filename
/// to its original source path. Called immediately after files are copied.
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

pub fn restore(backup_id: &str, user: &SteamUser) -> AppResult<usize> {
    let backup_dir = crate::paths::app_data_dir().join("backups").join(backup_id);

    if !backup_dir.exists() {
        return Err(AppError::Message(format!(
            "Backup '{backup_id}' not found."
        )));
    }

    let manifest = load_manifest(&backup_dir);
    if manifest.is_none() {
        debug!(
            backup_id,
            "No manifest found, falling back to filename inference"
        );
    }

    let mut restored = 0usize;
    for entry in fs::read_dir(&backup_dir).map_err(io_context(&backup_dir))? {
        let entry = entry.map_err(io_context(&backup_dir))?;
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

    info!(backup_id, restored, "Backup restored");
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

fn format_timestamp(id: &str) -> String {
    // Format: YYYYMMDD-HHMMSS
    if id.len() == 15 && id.as_bytes().get(8) == Some(&b'-') {
        format!(
            "{}-{}-{} {}:{}:{}",
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
