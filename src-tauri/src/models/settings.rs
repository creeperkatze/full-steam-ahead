use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UserSettings {
    pub stop_steam: bool,
    pub restart_steam: bool,
}
