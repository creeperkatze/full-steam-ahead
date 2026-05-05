use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamInstallation {
    pub install_path: PathBuf,
    pub users: Vec<SteamUser>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamUser {
    pub steam_id: String,
    pub account_name: Option<String>,
    pub shortcuts_path: PathBuf,
    pub grid_path: PathBuf,
    pub collections_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutEntry {
    pub app_id: u32,
    pub app_name: String,
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub shortcut_path: String,
    pub launch_options: String,
    pub is_hidden: bool,
    pub allow_desktop_config: bool,
    pub allow_overlay: bool,
    pub open_vr: bool,
    pub devkit: bool,
    pub devkit_game_id: String,
    pub last_play_time: u32,
    pub tags: Vec<String>,
}

impl Default for ShortcutEntry {
    fn default() -> Self {
        Self {
            app_id: 0,
            app_name: String::new(),
            exe: String::new(),
            start_dir: String::new(),
            icon: String::new(),
            shortcut_path: String::new(),
            launch_options: String::new(),
            is_hidden: false,
            allow_desktop_config: true,
            allow_overlay: true,
            open_vr: false,
            devkit: false,
            devkit_game_id: String::new(),
            last_play_time: 0,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualImportRequest {
    pub user_steam_id: String,
    pub executable_path: PathBuf,
    pub display_name: Option<String>,
    pub source: ImportSource,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub user_steam_id: String,
    pub include_playnite: bool,
    pub include_epic: bool,
    #[serde(default)]
    pub include_sources: Vec<ImportSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportSource {
    Manual,
    Playnite,
    Epic,
    Gog,
    Amazon,
    Bottles,
    Flatpak,
    GamePass,
    Heroic,
    Itch,
    Legendary,
    Lutris,
    MiniGalaxy,
    Origin,
    UbisoftConnect,
    Other(String),
}

impl ImportSource {
    pub fn display_name(&self) -> String {
        match self {
            ImportSource::Manual => "Manual".to_string(),
            ImportSource::Playnite => "Playnite".to_string(),
            ImportSource::Epic => "Epic Games".to_string(),
            ImportSource::Gog => "GOG".to_string(),
            ImportSource::Amazon => "Amazon Games".to_string(),
            ImportSource::Bottles => "Bottles".to_string(),
            ImportSource::Flatpak => "Flatpak".to_string(),
            ImportSource::GamePass => "Game Pass".to_string(),
            ImportSource::Heroic => "Heroic".to_string(),
            ImportSource::Itch => "itch.io".to_string(),
            ImportSource::Legendary => "Legendary".to_string(),
            ImportSource::Lutris => "Lutris".to_string(),
            ImportSource::MiniGalaxy => "MiniGalaxy".to_string(),
            ImportSource::Origin => "EA app / Origin".to_string(),
            ImportSource::UbisoftConnect => "Ubisoft Connect".to_string(),
            ImportSource::Other(value) => value.clone(),
        }
    }

    pub fn collection_name(&self) -> String {
        self.display_name()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub id: String,
    pub source: ImportSource,
    pub name: String,
    pub executable_path: PathBuf,
    pub start_dir: PathBuf,
    pub launch_options: Option<String>,
    pub existing_app_id: Option<u32>,
    pub matched_steam_app_id: Option<u32>,
    pub tags: Vec<String>,
    pub artwork: ArtworkPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkPlan {
    pub mode: ArtworkMode,
    pub existing: Vec<ArtworkAsset>,
    pub proposed: Vec<ArtworkAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkMode {
    PreserveExisting,
    OfficialSteamPreferred,
    SteamGridDbFallback,
    LocalOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkAsset {
    pub kind: ArtworkKind,
    pub path_or_url: String,
    pub source: ArtworkSource,
    pub will_replace_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkKind {
    Header,
    Capsule,
    Hero,
    Logo,
    Icon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkSource {
    ExistingCustom,
    OfficialSteam,
    SteamGridDb,
    LocalFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPlan {
    pub user_steam_id: String,
    pub changes: Vec<PlannedChange>,
    pub files_to_change: Vec<PathBuf>,
    pub backups: Vec<BackupPlan>,
    pub warnings: Vec<String>,
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
    pub title: String,
    pub file: PathBuf,
    pub kind: ChangeKind,
    pub destructive: bool,
    pub details: String,
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
    pub options: ApplyOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOptions {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub replace_existing_artwork: bool,
    pub write_collections: bool,
    pub use_legacy_collections_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub applied_changes: Vec<PlannedChange>,
    pub backups_created: Vec<PathBuf>,
    pub skipped_changes: Vec<String>,
}
