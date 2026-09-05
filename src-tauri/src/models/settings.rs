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
pub struct Settings {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub create_collections: bool,
    pub add_self_shortcut: bool,
    pub steam_location: Option<String>,
    #[serde(alias = "launchers")]
    pub sources: HashMap<String, SourceSettings>,
    pub steam_grid_db: SteamGridDbSettings,
    pub locale: Option<String>,
    pub color_scheme: Option<String>,
    pub update_notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            stop_steam: true,
            restart_steam: true,
            create_collections: true,
            add_self_shortcut: false,
            steam_location: None,
            sources: HashMap::new(),
            steam_grid_db: SteamGridDbSettings::default(),
            locale: None,
            color_scheme: None,
            update_notifications: true,
        }
    }
}

impl Settings {
    /// Returns the configured settings for an import source, or defaults if unset.
    pub fn source_settings(&self, source: &ImportSource) -> SourceSettings {
        source
            .settings_key()
            .and_then(|key| self.sources.get(key))
            .cloned()
            .unwrap_or_else(|| SourceSettings {
                enabled: !matches!(source, ImportSource::Flatpak),
                ..Default::default()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfigured_source_defaults_to_enabled() {
        let settings = Settings::default();
        assert!(settings.source_settings(&ImportSource::Gog).enabled);
    }

    #[test]
    fn unconfigured_flatpak_defaults_to_disabled() {
        let settings = Settings::default();
        assert!(!settings.source_settings(&ImportSource::Flatpak).enabled);
    }

    #[test]
    fn explicit_flatpak_setting_overrides_default() {
        let mut settings = Settings::default();
        settings.sources.insert(
            "flatpak".to_string(),
            SourceSettings {
                enabled: true,
                custom_path: None,
            },
        );
        assert!(settings.source_settings(&ImportSource::Flatpak).enabled);
    }
}
