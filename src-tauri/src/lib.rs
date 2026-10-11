mod backups;
mod cli;
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
    window_corners_rounded,
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

    // Commands print their own output, so the log would only clutter it.
    let stderr_layer = if cfg!(debug_assertions) && !cli::requested() {
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

/// Every command and event the frontend can use. The TypeScript bindings are generated from it.
fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
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
            window_corners_rounded,
            list_backups,
            restore_backup,
            delete_backup,
            delete_all_backups,
            get_debug_info,
            open_logs_folder,
        ])
        .events(tauri_specta::collect_events![
            models::ScanProgressEvent,
            models::ApplyProgressEvent
        ])
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        // Counts and file sizes stay far below the largest safe JavaScript number.
        .dangerously_cast_bigints_to_number()
}

/// Writes `src/bindings.ts`. Only debug builds run from the source tree, so release builds skip it.
#[cfg(debug_assertions)]
fn export_bindings(
    builder: &tauri_specta::Builder<tauri::Wry>,
) -> Result<(), specta_typescript::Error> {
    builder.export(
        specta_typescript::Typescript::default(),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts"),
    )
}

/// Builds the main window with a GTK frame for its shadow and resize edges.
#[cfg(target_os = "linux")]
fn create_linux_main_window(app: &tauri::App) -> tauri::Result<()> {
    use gtk::prelude::{GtkWindowExt, StyleContextExt, WidgetExt};

    let Some(config) = app.config().app.windows.iter().find(|w| w.label == "main") else {
        return Ok(());
    };

    // Rounded corners need transparency, which needs a compositor
    let screen = gtk::gdk::Screen::default().filter(|screen| screen.is_composited());
    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), config)?
        .transparent(screen.is_some());
    if let Some(screen) = &screen {
        style_gtk_frame(screen);
        builder = builder.initialization_script("window.__FSA_ROUNDED_CORNERS__ = true;");
    }

    // An empty titlebar keeps the frame but hides the title bar
    let gtk_window = builder.build()?.gtk_window()?;
    gtk_window.set_titlebar(Some(&gtk::EventBox::new()));
    // Scopes the frame style to this window, so file dialogs keep the theme's frame
    gtk_window.style_context().add_class("fsa-main");
    gtk_window.connect_window_state_event(|_, event| {
        use gtk::gdk::WindowState;
        let square = WindowState::MAXIMIZED
            | WindowState::FULLSCREEN
            | WindowState::TILED
            | WindowState::TOP_TILED
            | WindowState::RIGHT_TILED
            | WindowState::BOTTOM_TILED
            | WindowState::LEFT_TILED;
        let rounded = !event.new_window_state().intersects(square);
        commands::WINDOW_CORNERS_ROUNDED.store(rounded, std::sync::atomic::Ordering::Relaxed);
        gtk::glib::Propagation::Proceed
    });
    Ok(())
}

/// Gives the frame the radius and shadows of libadwaita windows.
#[cfg(target_os = "linux")]
fn style_gtk_frame(screen: &gtk::gdk::Screen) {
    use gtk::prelude::CssProviderExt;

    // The radius matches the page body in main.css
    const CSS: &[u8] = b"
        .fsa-main decoration {
            border-radius: 15px;
            margin: 20px;
            box-shadow: 0 0 14px 5px rgba(0, 0, 0, 0.15), 0 0 5px 2px rgba(0, 0, 0, 0.1),
                0 0 0 1px rgba(0, 0, 0, 0.05);
        }
        .fsa-main decoration:backdrop {
            box-shadow: 0 0 14px 5px transparent, 0 0 10px 5px rgba(0, 0, 0, 0.08),
                0 0 0 1px rgba(0, 0, 0, 0.05);
        }
        .fsa-main.maximized decoration, .fsa-main.fullscreen decoration,
        .fsa-main.tiled decoration, .fsa-main.tiled-top decoration,
        .fsa-main.tiled-right decoration, .fsa-main.tiled-bottom decoration,
        .fsa-main.tiled-left decoration {
            border-radius: 0;
            box-shadow: 0 0 0 20px transparent;
        }
    ";
    let provider = gtk::CssProvider::new();
    if let Err(error) = provider.load_from_data(CSS) {
        tracing::warn!(%error, "Could not load the window frame style");
        return;
    }
    gtk::StyleContext::add_provider_for_screen(
        screen,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta = specta_builder();
    #[cfg(debug_assertions)]
    if std::env::args().any(|arg| arg == "--export-bindings") {
        export_bindings(&specta).expect("Could not export the TypeScript bindings");
        return;
    }

    let log_guard = init_logging();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        "Full Steam Ahead starting"
    );

    if let Some(code) = cli::run() {
        // Exiting skips destructors, so flush the log first.
        drop(log_guard);
        std::process::exit(code);
    }

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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_wayland_nvidia_quirk::init());

    // Flatpak apps update through Flathub, not the built-in updater.
    #[cfg(not(feature = "disable-auto-updates"))]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    #[cfg(debug_assertions)]
    if let Err(error) = export_bindings(&specta) {
        tracing::warn!(%error, "Could not export the TypeScript bindings");
    }
    let invoke_handler = specta.invoke_handler();

    builder
        .setup(move |app| {
            specta.mount_events(app);
            #[cfg(target_os = "linux")]
            create_linux_main_window(app)?;
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
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running Full Steam Ahead");
}
