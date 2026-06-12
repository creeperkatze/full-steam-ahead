use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamInstallation {
    pub install_path: PathBuf,
    pub users: Vec<SteamUser>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamUser {
    pub steam_id: String,
    pub account_name: Option<String>,
    pub avatar_path: Option<PathBuf>,
    pub shortcuts_path: PathBuf,
    pub grid_path: PathBuf,
    pub collections_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutEntry {
    pub app_id: u32,
    pub app_name: String,
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub shortcut_path: String,
    pub launch_options: String,
    pub is_hidden: bool,
    pub allow_desktop_config: bool,
    pub allow_overlay: bool,
    pub open_vr: bool,
    pub devkit: bool,
    pub devkit_game_id: String,
    pub last_play_time: u32,
    pub tags: Vec<String>,
}

impl Default for ShortcutEntry {
    fn default() -> Self {
        Self {
            app_id: 0,
            app_name: String::new(),
            exe: String::new(),
            start_dir: String::new(),
            icon: String::new(),
            shortcut_path: String::new(),
            launch_options: String::new(),
            is_hidden: false,
            allow_desktop_config: true,
            allow_overlay: true,
            open_vr: false,
            devkit: false,
            devkit_game_id: String::new(),
            last_play_time: 0,
            tags: Vec::new(),
        }
    }
}
