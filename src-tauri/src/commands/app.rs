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

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub fn window_corners_rounded(window: tauri::WebviewWindow) -> bool {
    // Stays sync, because GTK only works on the main thread
    #[cfg(target_os = "linux")]
    {
        use gtk::{gdk::WindowState, prelude::WidgetExt};

        let square = WindowState::MAXIMIZED
            | WindowState::FULLSCREEN
            | WindowState::TILED
            | WindowState::TOP_TILED
            | WindowState::RIGHT_TILED
            | WindowState::BOTTOM_TILED
            | WindowState::LEFT_TILED;
        window
            .gtk_window()
            .ok()
            .and_then(|gtk_window| gtk_window.window())
            .is_some_and(|gdk_window| !gdk_window.state().intersects(square))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        false
    }
}
