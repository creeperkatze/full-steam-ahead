use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserSettings {
    pub stop_steam: bool,
    pub restart_steam: bool,
    pub create_collections: bool,
    pub steam_location: Option<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            stop_steam: true,
            restart_steam: true,
            create_collections: true,
            steam_location: None,
        }
    }
}
