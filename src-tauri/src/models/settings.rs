use super::importers::ImportSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LauncherSettings {
    pub enabled: bool,
    pub custom_path: Option<String>,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            custom_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserSettings {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub create_collections: bool,
    pub steam_location: Option<String>,
    pub launchers: HashMap<String, LauncherSettings>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            stop_steam: true,
            restart_steam: true,
            create_collections: true,
            steam_location: None,
            launchers: HashMap::new(),
        }
    }
}

impl UserSettings {
    /// Returns the configured settings for a launcher, or defaults (enabled, auto-detected) if unset.
    pub fn launcher(&self, source: &ImportSource) -> LauncherSettings {
        source
            .settings_key()
            .and_then(|key| self.launchers.get(key))
            .cloned()
            .unwrap_or_default()
    }
}
