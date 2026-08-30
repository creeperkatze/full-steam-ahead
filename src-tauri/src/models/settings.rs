use super::importers::ImportSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SourceSettings {
    pub enabled: bool,
    pub custom_path: Option<String>,
}

impl Default for SourceSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            custom_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SteamGridDbSettings {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub allow_nsfw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserSettings {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub create_collections: bool,
    pub steam_location: Option<String>,
    #[serde(alias = "launchers")]
    pub sources: HashMap<String, SourceSettings>,
    pub steam_grid_db: SteamGridDbSettings,
    pub locale: Option<String>,
    pub color_scheme: Option<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            stop_steam: true,
            restart_steam: true,
            create_collections: true,
            steam_location: None,
            sources: HashMap::new(),
            steam_grid_db: SteamGridDbSettings::default(),
            locale: None,
            color_scheme: None,
        }
    }
}

impl UserSettings {
    /// Returns the configured settings for an import source, or defaults (enabled, auto-detected) if unset.
    pub fn source_settings(&self, source: &ImportSource) -> SourceSettings {
        source
            .settings_key()
            .and_then(|key| self.sources.get(key))
            .cloned()
            .unwrap_or_default()
    }
}
