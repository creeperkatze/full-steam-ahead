use crate::{
    backups,
    error::{io_context, AppError, CommandError},
    models::{
        ApplyRequest, ApplyResult, BackupInfo, ImportCandidate, ManualImportRequest, Options,
        PreviewPlan, ScanRequest, ShortcutEntry, SteamInstallation, UserSettings,
    },
    steam,
};
use chrono::Utc;
use std::{fs, path::PathBuf};
use tracing::{debug, info, instrument, warn};

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
pub fn detect_steam() -> CommandResult<SteamInstallation> {
    let result = steam::detect::detect_steam().map_err(Into::into);
    if let Ok(ref install) = result {
        info!(
            users = install.users.len(),
            running = install.running,
            "Steam detected"
        );
    }
    result
}

#[tauri::command]
#[instrument]
pub fn read_shortcuts_for_user(user_steam_id: String) -> CommandResult<Vec<ShortcutEntry>> {
    let user = steam::detect::find_user(&user_steam_id)?;
    let result = steam::shortcuts::read_shortcuts(&user.shortcuts_path).map_err(Into::into);
    if let Ok(ref shortcuts) = result {
        debug!(count = shortcuts.len(), "Shortcuts loaded");
    }
    result
}

#[tauri::command]
#[instrument(skip(app), fields(user = %request.user_steam_id, sources = request.include_sources.len()))]
pub fn scan_sources(
    app: tauri::AppHandle,
    request: ScanRequest,
) -> CommandResult<Vec<ImportCandidate>> {
    let user = steam::detect::find_user(&request.user_steam_id)?;
    let result =
        steam::sources::scan_sources_with_progress(&app, &user, &request).map_err(Into::into);
    if let Ok(ref candidates) = result {
        info!(total = candidates.len(), "Scan complete");
    }
    result
}

#[tauri::command]
#[instrument(skip(candidates, options), fields(user = %user_steam_id, candidates = candidates.len()))]
pub fn create_preview_plan(
    user_steam_id: String,
    candidates: Vec<ImportCandidate>,
    options: Options,
) -> CommandResult<PreviewPlan> {
    let user = steam::detect::find_user(&user_steam_id)?;
    let backup_root = crate::paths::app_data_dir()
        .join("backups")
        .join(Utc::now().format("%Y%m%d-%H%M%S").to_string());

    let plan = steam::plan::build_preview_plan(&user, &candidates, &options, &backup_root)?;
    info!(
        changes = plan.changes.len(),
        backups = plan.backups.len(),
        "Preview plan created"
    );
    Ok(plan)
}

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

#[tauri::command]
#[instrument(fields(name = ?request.display_name, exe = %request.executable_path.display()))]
pub fn create_manual_candidate(request: ManualImportRequest) -> CommandResult<ImportCandidate> {
    let user = steam::detect::find_user(&request.user_steam_id)?;
    Ok(crate::importers::manual::candidate_with_grid_path(
        request,
        &user.grid_path,
    ))
}

#[tauri::command]
#[instrument(skip(app, request), fields(user = %request.plan.user_steam_id, candidates = request.candidates.len()))]
pub fn apply_plan(app: tauri::AppHandle, request: ApplyRequest) -> CommandResult<ApplyResult> {
    let result = steam::apply::apply_plan_with_progress(&app, request).map_err(Into::into);
    if let Ok(ref r) = result {
        info!(
            applied = r.applied_changes.len(),
            backups = r.backups_created.len(),
            "Plan applied"
        );
    }
    result
}

#[tauri::command]
#[instrument(skip_all)]
pub fn close_app(app: tauri::AppHandle) {
    info!("Application closing");
    app.exit(0);
}

#[tauri::command]
#[instrument]
pub fn list_backups() -> CommandResult<Vec<BackupInfo>> {
    backups::list().map_err(Into::into)
}

#[tauri::command]
#[instrument]
pub fn restore_backup(backup_id: String) -> CommandResult<usize> {
    let restored = backups::restore_backup(&backup_id)?;
    info!(backup_id, restored, "Backup restored via command");
    Ok(restored)
}
