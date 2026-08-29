use crate::{
    error::{AppError, CommandError},
    paths,
};
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tracing::{info, instrument};

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInfo {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub data_path: String,
}

fn format_os(os: &str) -> String {
    match os {
        "windows" => "Windows".to_string(),
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        other => other.to_string(),
    }
}

#[tauri::command]
#[instrument]
pub fn get_debug_info() -> CommandResult<DebugInfo> {
    Ok(DebugInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: format_os(std::env::consts::OS),
        arch: std::env::consts::ARCH.to_string(),
        data_path: paths::app_data_dir().display().to_string(),
    })
}

#[tauri::command]
#[instrument(skip(app))]
pub fn open_logs_folder(app: AppHandle) -> CommandResult<()> {
    let logs_dir = paths::logs_dir();
    app.opener()
        .open_path(logs_dir.display().to_string(), None::<&str>)
        .map_err(|e| AppError::Message(e.to_string()))?;
    info!("Opened logs folder");
    Ok(())
}
