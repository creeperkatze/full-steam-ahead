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
    // Flatpak defaults to off since it lists every installed flatpak, not just games.
    fn default_source_settings(source: &ImportSource) -> SourceSettings {
        SourceSettings {
            enabled: !matches!(source, ImportSource::Flatpak),
            custom_path: None,
        }
    }

    pub fn source_settings(&self, source: &ImportSource) -> SourceSettings {
        source
            .settings_key()
            .and_then(|key| self.sources.get(key))
            .cloned()
            .unwrap_or_else(|| Self::default_source_settings(source))
    }

    // Backfills missing entries so callers never have to guess a source's default.
    pub fn ensure_source_defaults(&mut self, sources: &[ImportSource]) {
        for source in sources {
            if let Some(key) = source.settings_key() {
                self.sources
                    .entry(key.to_string())
                    .or_insert_with(|| Self::default_source_settings(source));
            }
        }
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

    #[test]
    fn ensure_source_defaults_fills_in_missing_entries() {
        let mut settings = Settings::default();
        settings.ensure_source_defaults(&[ImportSource::Gog, ImportSource::Flatpak]);

        assert!(settings.sources["gog"].enabled);
        assert!(!settings.sources["flatpak"].enabled);
    }

    #[test]
    fn ensure_source_defaults_does_not_override_existing_entries() {
        let mut settings = Settings::default();
        settings.sources.insert(
            "flatpak".to_string(),
            SourceSettings {
                enabled: true,
                custom_path: None,
            },
        );
        settings.ensure_source_defaults(&[ImportSource::Flatpak]);

        assert!(settings.sources["flatpak"].enabled);
    }
}
