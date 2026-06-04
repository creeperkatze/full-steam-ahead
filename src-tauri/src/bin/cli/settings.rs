use full_steam_ahead_lib::{models::UserSettings, paths};
use std::fs;

pub fn load_quietly() -> UserSettings {
    let path = paths::app_data_dir().join("settings.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}
