use super::importers::ImportSource;
use serde::Serialize;
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub source: ImportSource,
    pub status: ScanStatus,
    pub found: usize,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus {
    Scanning,
    Done,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum ApplyStep {
    StoppingSteam,
    CreatingBackups,
    ApplyingArtwork { game_name: Option<String> },
    UpdatingShortcuts,
    UpdatingCollections,
    RestartingSteam,
}

#[derive(Debug, Clone, Serialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ApplyProgressEvent {
    pub step: ApplyStep,
    pub current: usize,
    pub total: usize,
}
