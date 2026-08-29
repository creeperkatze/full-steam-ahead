use crate::{
    error::{io_context, AppError, CommandError},
    models::{ImportSource, UserSettings},
    paths, steam,
};
use std::fs;
use tracing::instrument;

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
pub fn available_sources() -> Vec<ImportSource> {
    steam::sources::scannable_sources()
}

#[tauri::command]
#[instrument(skip_all)]
pub fn load_settings() -> CommandResult<UserSettings> {
    let path = paths::settings_path();
    if !path.exists() {
        return Ok(UserSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

#[tauri::command]
#[instrument(skip_all)]
pub fn save_settings(settings: UserSettings) -> CommandResult<()> {
    let path = paths::settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    let raw = serde_json::to_string_pretty(&settings)
        .map_err(|_| AppError::Message("Failed to serialize settings.".to_string()))?;
    fs::write(&path, raw).map_err(io_context(&path))?;
    Ok(())
}
