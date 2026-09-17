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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum DefaultArtworkSource {
    None,
    #[default]
    Steam,
    SteamGridDb,
}

/// Whether games start through their launcher or from their own executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LauncherMode {
    Always,
    #[default]
    WhenRequired,
    WhenNoExecutable,
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
    pub default_artwork_source: DefaultArtworkSource,
    pub launcher_mode: LauncherMode,
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
            default_artwork_source: DefaultArtworkSource::default(),
            launcher_mode: LauncherMode::default(),
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
        self.migrate_renamed_sources();
        for source in sources {
            if let Some(key) = source.settings_key() {
                self.sources
                    .entry(key.to_string())
                    .or_insert_with(|| Self::default_source_settings(source));
            }
        }
    }
}

impl Settings {
    /// Moves settings saved under a source's old key to its current key.
    fn migrate_renamed_sources(&mut self) {
        const RENAMED: [(&str, &str); 1] = [("origin", "eaApp")];
        for (old, new) in RENAMED {
            if let Some(settings) = self.sources.remove(old) {
                self.sources.entry(new.to_string()).or_insert(settings);
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
    fn origin_settings_move_to_ea_app() {
        let mut settings: Settings = serde_json::from_str(
            r#"{"sources": {"origin": {"enabled": false, "customPath": "D:/EA"}}}"#,
        )
        .unwrap();
        settings.ensure_source_defaults(&[ImportSource::EaApp]);
        assert!(!settings.sources.contains_key("origin"));
        let ea_app = settings.source_settings(&ImportSource::EaApp);
        assert!(!ea_app.enabled);
        assert_eq!(ea_app.custom_path.as_deref(), Some("D:/EA"));
    }

    #[test]
    fn current_ea_app_settings_win_over_old_ones() {
        let mut settings = Settings::default();
        for (key, enabled) in [("origin", false), ("eaApp", true)] {
            settings.sources.insert(
                key.to_string(),
                SourceSettings {
                    enabled,
                    custom_path: None,
                },
            );
        }
        settings.ensure_source_defaults(&[ImportSource::EaApp]);
        assert!(settings.source_settings(&ImportSource::EaApp).enabled);
        assert!(!settings.sources.contains_key("origin"));
    }

    #[test]
    fn origin_source_id_is_still_accepted() {
        let source: ImportSource = serde_json::from_str(r#""origin""#).unwrap();
        assert_eq!(source, ImportSource::EaApp);
        assert_eq!(serde_json::to_string(&source).unwrap(), r#""eaApp""#);
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
