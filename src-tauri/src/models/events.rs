use super::importers::ImportSource;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub source: ImportSource,
    pub status: String,
    pub found: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ApplyStep {
    StoppingSteam,
    CreatingBackups,
    ApplyingArtwork { game_name: Option<String> },
    UpdatingShortcuts,
    UpdatingCollections,
    RestartingSteam,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyProgressEvent {
    pub step: ApplyStep,
    pub current: usize,
    pub total: usize,
}
