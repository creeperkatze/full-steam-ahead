use super::settings::LauncherMode;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    #[serde(alias = "origin")]
    EaApp,
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
            ImportSource::EaApp => "EA app".to_string(),
            ImportSource::UbisoftConnect => "Ubisoft Connect".to_string(),
            ImportSource::Other(value) => value.clone(),
        }
    }

    pub fn collection_name(&self) -> String {
        self.display_name()
    }

    pub fn settings_key(&self) -> Option<&'static str> {
        match self {
            ImportSource::Playnite => Some("playnite"),
            ImportSource::Epic => Some("epic"),
            ImportSource::Gog => Some("gog"),
            ImportSource::Amazon => Some("amazon"),
            ImportSource::Bottles => Some("bottles"),
            ImportSource::Flatpak => Some("flatpak"),
            ImportSource::GamePass => Some("gamePass"),
            ImportSource::Heroic => Some("heroic"),
            ImportSource::Itch => Some("itch"),
            ImportSource::Legendary => Some("legendary"),
            ImportSource::Lutris => Some("lutris"),
            ImportSource::MiniGalaxy => Some("miniGalaxy"),
            ImportSource::EaApp => Some("eaApp"),
            ImportSource::UbisoftConnect => Some("ubisoftConnect"),
            ImportSource::Manual | ImportSource::Other(_) => None,
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
    #[serde(default)]
    pub include_sources: Vec<ImportSource>,
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
    pub url_scheme: Option<String>,
    pub launcher_path: Option<PathBuf>,
    pub use_launcher_url: bool,
    #[serde(default)]
    pub needs_proton: bool,
}

impl ImportCandidate {
    pub fn effective_executable(&self) -> &Path {
        if self.use_launcher_url {
            self.launcher_path
                .as_deref()
                .unwrap_or(&self.executable_path)
        } else {
            &self.executable_path
        }
    }

    pub fn effective_start_dir(&self) -> &Path {
        if self.use_launcher_url {
            self.launcher_path
                .as_deref()
                .and_then(|p| p.parent())
                .unwrap_or(&self.start_dir)
        } else {
            &self.start_dir
        }
    }

    pub fn effective_launch_options(&self) -> Option<&str> {
        if self.use_launcher_url {
            self.url_scheme
                .as_deref()
                .or(self.launch_options.as_deref())
        } else {
            self.launch_options.as_deref()
        }
    }

    /// Applies the launcher preference, skipping candidates that only start one way.
    pub fn apply_launcher_mode(&mut self, mode: LauncherMode) {
        if self.launcher_path.is_none() || self.url_scheme.is_none() {
            return;
        }
        match mode {
            LauncherMode::Always => self.use_launcher_url = true,
            // The importer's own default stands.
            LauncherMode::WhenRequired => {}
            LauncherMode::WhenNoExecutable => self.use_launcher_url = false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkPlan {
    pub mode: ArtworkMode,
    pub existing: Vec<ArtworkAsset>,
    pub proposed: Vec<ArtworkAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl ArtworkKind {
    pub fn slug(&self) -> &'static str {
        match self {
            ArtworkKind::Header => "header",
            ArtworkKind::Capsule => "capsule",
            ArtworkKind::Hero => "hero",
            ArtworkKind::Logo => "logo",
            ArtworkKind::Icon => "icon",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtworkSource {
    ExistingCustom,
    OfficialSteam,
    SteamGridDb,
    LocalFile,
    Missing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toggleable() -> ImportCandidate {
        ImportCandidate {
            id: "epic-1".to_string(),
            source: ImportSource::Epic,
            name: "Test Game".to_string(),
            executable_path: PathBuf::from("/games/test/game.exe"),
            start_dir: PathBuf::from("/games/test"),
            launch_options: None,
            existing_app_id: None,
            matched_steam_app_id: None,
            tags: Vec::new(),
            artwork: ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing: Vec::new(),
                proposed: Vec::new(),
            },
            url_scheme: Some("com.epicgames.launcher://apps/x".to_string()),
            launcher_path: Some(PathBuf::from("/epic/EpicGamesLauncher.exe")),
            use_launcher_url: true,
            needs_proton: false,
        }
    }

    // No executable of its own, so the launcher is the only way in.
    fn launcher_only() -> ImportCandidate {
        ImportCandidate {
            launcher_path: None,
            ..toggleable()
        }
    }

    #[test]
    fn always_routes_a_toggleable_candidate_through_the_launcher() {
        let mut c = toggleable();
        c.use_launcher_url = false;
        c.apply_launcher_mode(LauncherMode::Always);
        assert!(c.use_launcher_url);
    }

    #[test]
    fn when_no_executable_prefers_the_executable() {
        let mut c = toggleable();
        c.apply_launcher_mode(LauncherMode::WhenNoExecutable);
        assert!(!c.use_launcher_url);
        assert_eq!(c.effective_executable(), Path::new("/games/test/game.exe"));
    }

    #[test]
    fn when_required_leaves_the_importer_default_alone() {
        for importer_default in [true, false] {
            let mut c = toggleable();
            c.use_launcher_url = importer_default;
            c.apply_launcher_mode(LauncherMode::WhenRequired);
            assert_eq!(c.use_launcher_url, importer_default);
        }
    }

    #[test]
    fn launcher_only_candidates_ignore_every_mode() {
        for mode in [
            LauncherMode::Always,
            LauncherMode::WhenRequired,
            LauncherMode::WhenNoExecutable,
        ] {
            let mut c = launcher_only();
            c.apply_launcher_mode(mode);
            assert!(c.use_launcher_url, "{mode:?}");
        }
    }
}
