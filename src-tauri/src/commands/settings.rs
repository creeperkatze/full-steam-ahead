use crate::{
    error::{io_context, AppError, CommandError},
    importers,
    models::{ImportSource, Settings},
    paths,
};
use std::{fs, path::Path};
use tracing::instrument;

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn available_sources() -> Vec<ImportSource> {
    importers::scan::scannable_sources()
}

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub fn load_settings() -> CommandResult<Settings> {
    let path = paths::settings_path();
    let mut settings: Settings = if !path.exists() {
        Settings::default()
    } else {
        let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
        Settings::from_json(&raw).unwrap_or_else(|error| {
            tracing::warn!(path = %path.display(), %error, "Settings file is invalid, using defaults");
            Settings::default()
        })
    };
    settings.ensure_source_defaults(&importers::scan::scannable_sources());
    Ok(settings)
}

#[tauri::command]
#[instrument(skip_all)]
#[specta::specta]
pub fn save_settings(settings: Settings) -> CommandResult<()> {
    let path = paths::settings_path();
    write_settings(&path, &settings)?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(settings))]
#[specta::specta]
pub fn export_settings(path: String, settings: Settings) -> CommandResult<()> {
    write_settings(Path::new(&path), &settings)?;
    Ok(())
}

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn import_settings(path: String) -> CommandResult<Settings> {
    let path = Path::new(&path);
    let raw = fs::read_to_string(path).map_err(io_context(path))?;
    let mut settings = Settings::from_json(&raw).map_err(|source| AppError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    settings.ensure_source_defaults(&importers::scan::scannable_sources());
    write_settings(&paths::settings_path(), &settings)?;
    Ok(settings)
}

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn reset_settings() -> CommandResult<Settings> {
    let path = paths::settings_path();
    let mut settings = Settings::default();
    settings.ensure_source_defaults(&importers::scan::scannable_sources());
    write_settings(&path, &settings)?;
    Ok(settings)
}

fn write_settings(path: &Path, settings: &Settings) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|_| AppError::Message("Failed to serialize settings.".to_string()))?;
    fs::write(path, raw).map_err(io_context(path))
}
