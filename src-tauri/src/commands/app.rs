use std::sync::atomic::{AtomicBool, Ordering};
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

/// Kept up to date by the Linux main window, whose frame turns square when maximized or tiled.
pub static WINDOW_CORNERS_ROUNDED: AtomicBool = AtomicBool::new(true);

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub async fn window_corners_rounded() -> bool {
    WINDOW_CORNERS_ROUNDED.load(Ordering::Relaxed)
}
