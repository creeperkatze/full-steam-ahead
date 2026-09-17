mod backups;
mod commands;
pub mod error;
mod importers;
pub mod models;
pub mod paths;
mod process;
pub mod steam;
mod util;

use commands::{
    apply_plan, available_sources, close_app, create_manual_candidate, create_preview_plan,
    delete_all_backups, delete_backup, detect_steam, export_settings, get_debug_info,
    grant_steam_flatpak_permission, import_settings, list_backups, load_settings, open_logs_folder,
    read_shortcuts_for_user, reset_settings, restore_backup, save_settings, scan_sources,
    show_main_window, steamgriddb_images, steamgriddb_search, validate_steam_location,
};
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() -> WorkerGuard {
    let log_dir = paths::logs_dir();

    let session_filename = format!(
        "session_{}.log",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let file_appender = tracing_appender::rolling::never(log_dir, session_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let default_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("full_steam_ahead_lib={default_level}")));

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

    log_panics();
    guard
}

/// Records panics in the session log before the default hook prints them.
fn log_panics() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        tracing::error!(%info, %backtrace, "Panic");
        default_hook(info);
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _log_guard = init_logging();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        "Full Steam Ahead starting"
    );

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(error) = window.unminimize().and_then(|()| window.set_focus()) {
                    tracing::warn!(%error, "Could not focus the existing window");
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init());

    // Flatpak apps update through Flathub, not the built-in updater.
    #[cfg(not(feature = "disable-auto-updates"))]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .setup(|app| {
            #[cfg(not(target_os = "linux"))]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_shadow(true) {
                    tracing::warn!(error = %e, "Failed to set window shadow");
                }
            }

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
            validate_steam_location,
            grant_steam_flatpak_permission,
            read_shortcuts_for_user,
            scan_sources,
            create_manual_candidate,
            create_preview_plan,
            apply_plan,
            load_settings,
            save_settings,
            export_settings,
            import_settings,
            reset_settings,
            available_sources,
            steamgriddb_search,
            steamgriddb_images,
            close_app,
            show_main_window,
            list_backups,
            restore_backup,
            delete_backup,
            delete_all_backups,
            get_debug_info,
            open_logs_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Full Steam Ahead");
}
