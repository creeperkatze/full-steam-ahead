mod commands;
mod error;
mod importers;
mod models;
mod process;
mod steam;

use commands::{
    apply_plan, close_app, create_manual_candidate, create_preview_plan, detect_steam,
    load_settings, read_shortcuts_for_user, save_settings, scan_sources,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect_steam,
            read_shortcuts_for_user,
            scan_sources,
            create_manual_candidate,
            create_preview_plan,
            apply_plan,
            load_settings,
            save_settings,
            close_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Full Steam Ahead");
}
