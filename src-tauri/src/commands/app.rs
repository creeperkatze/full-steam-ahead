use tauri::Manager;
use tracing::{info, instrument};

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub fn close_app(app: tauri::AppHandle) {
    info!("Application closing");
    app.exit(0);
}

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub fn show_main_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.show() {
            tracing::warn!(%error, "Could not show the main window");
        }
    }
}
