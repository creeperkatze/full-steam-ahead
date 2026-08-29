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
    pub data_dir: String,
    pub logs_dir: String,
    pub backups_dir: String,
    pub settings_path: String,
}

#[tauri::command]
#[instrument]
pub fn get_debug_info() -> CommandResult<DebugInfo> {
    Ok(DebugInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: paths::app_data_dir().display().to_string(),
        logs_dir: paths::logs_dir().display().to_string(),
        backups_dir: paths::backups_dir().display().to_string(),
        settings_path: paths::settings_path().display().to_string(),
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
