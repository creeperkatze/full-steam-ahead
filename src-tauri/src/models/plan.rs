use super::importers::{ArtworkKind, ArtworkSource, ImportCandidate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPlan {
    pub user_steam_id: String,
    pub changes: Vec<PlannedChange>,
    pub files_to_change: Vec<PathBuf>,
    pub backups: Vec<BackupPlan>,
    pub requires_steam_restart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlan {
    pub source: PathBuf,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedChange {
    pub id: String,
    pub game_name: String,
    pub file: PathBuf,
    pub kind: ChangeKind,
    pub destructive: bool,
    pub artwork_source: Option<ArtworkSource>,
    pub artwork_kind: Option<ArtworkKind>,
    pub collection_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    AddShortcut,
    UpdateShortcut,
    WriteArtwork,
    UpdateCollections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRequest {
    pub plan: PreviewPlan,
    pub candidates: Vec<ImportCandidate>,
    pub options: Options,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub replace_existing_artwork: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub applied_changes: Vec<PlannedChange>,
    pub backups_created: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub file_count: usize,
    pub size_bytes: u64,
}
