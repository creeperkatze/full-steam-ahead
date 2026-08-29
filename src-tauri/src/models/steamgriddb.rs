use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGridDbGame {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamGridDbImage {
    pub id: u32,
    pub url: String,
    pub thumbnail_url: String,
    pub width: u32,
    pub height: u32,
}
