use crate::{
    error::{io_context, AppError, CommandError},
    models::UserSettings,
};
use std::{fs, path::PathBuf};
use tracing::instrument;

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument(skip_all)]
pub fn load_settings() -> CommandResult<UserSettings> {
    let path = settings_path();
    if !path.exists() {
        return Ok(UserSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

#[tauri::command]
#[instrument(skip_all)]
pub fn save_settings(settings: UserSettings) -> CommandResult<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    let raw = serde_json::to_string_pretty(&settings)
        .map_err(|_| AppError::Message("Failed to serialize settings.".to_string()))?;
    fs::write(&path, raw).map_err(io_context(&path))?;
    Ok(())
}

fn settings_path() -> PathBuf {
    crate::paths::app_data_dir().join("settings.json")
}
