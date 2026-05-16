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
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() -> WorkerGuard {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Full Steam Ahead")
        .join("logs");

    let file_appender = tracing_appender::rolling::daily(log_dir, "full-steam-ahead.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("full_steam_ahead_lib=info"));

    let stderr_layer = if cfg!(debug_assertions) {
        Some(fmt::layer().with_writer(std::io::stderr))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(stderr_layer)
        .with(tracing_error::ErrorLayer::default())
        .init();

    guard
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _log_guard = init_logging();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Full Steam Ahead starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Ok(install) = steam::detect::detect_steam() {
                if let Err(e) = app.asset_protocol_scope().allow_directory(&install.install_path, true) {
                    tracing::warn!(error = %e, path = %install.install_path.display(), "Could not extend asset scope for Steam path");
                } else {
                    tracing::debug!(path = %install.install_path.display(), "Steam path added to asset scope");
                }
            }
            Ok(())
        })
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
