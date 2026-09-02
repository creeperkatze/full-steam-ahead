use crate::{
    error::{io_context, AppError, CommandError},
    models::{ImportSource, Settings},
    paths, steam,
};
use std::{fs, path::Path};
use tracing::instrument;

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
pub fn available_sources() -> Vec<ImportSource> {
    steam::sources::scannable_sources()
}

#[tauri::command]
#[instrument(skip_all)]
pub fn load_settings() -> CommandResult<Settings> {
    let path = paths::settings_path();
    if !path.exists() {
        return Ok(Settings::default());
    }
    let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

#[tauri::command]
#[instrument(skip_all)]
pub fn save_settings(settings: Settings) -> CommandResult<()> {
    let path = paths::settings_path();
    write_settings(&path, &settings)?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(settings))]
pub fn export_settings(path: String, settings: Settings) -> CommandResult<()> {
    write_settings(Path::new(&path), &settings)?;
    Ok(())
}

#[tauri::command]
#[instrument]
pub fn import_settings(path: String) -> CommandResult<Settings> {
    let path = Path::new(&path);
    let raw = fs::read_to_string(path).map_err(io_context(path))?;
    let settings = serde_json::from_str(&raw).map_err(|source| AppError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(settings)
}

#[tauri::command]
#[instrument]
pub fn reset_settings() -> CommandResult<Settings> {
    let path = paths::settings_path();
    if path.exists() {
        fs::remove_file(&path).map_err(io_context(&path))?;
    }
    Ok(Settings::default())
}

fn write_settings(path: &Path, settings: &Settings) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|_| AppError::Message("Failed to serialize settings.".to_string()))?;
    fs::write(path, raw).map_err(io_context(path))
}
