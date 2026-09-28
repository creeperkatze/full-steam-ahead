use crate::{
    error::CommandError,
    models::{
        ApplyRequest, ApplyResult, ImportCandidate, ManualImportRequest, PreviewPlan, ScanRequest,
        ShortcutEntry, SteamInstallation,
    },
    steam,
};
use tauri_specta::Event;
use tracing::{debug, info, instrument};

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn detect_steam() -> CommandResult<SteamInstallation> {
    let result = steam::detect::detect_steam().map_err(Into::into);
    if let Ok(ref install) = result {
        info!(
            path = %install.install_path.display(),
            users = install.users.len(),
            running = install.running,
            "Steam detected"
        );
    }
    result
}

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn validate_steam_location(path: String) -> bool {
    steam::detect::is_valid_steam_location(std::path::Path::new(&path))
}

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn grant_steam_flatpak_permission() -> CommandResult<()> {
    #[cfg(unix)]
    {
        steam::detect::grant_steam_flatpak_permission().map_err(Into::into)
    }
    #[cfg(not(unix))]
    {
        Ok(())
    }
}

#[tauri::command]
#[instrument]
#[specta::specta]
pub fn read_shortcuts_for_user(user_steam_id: String) -> CommandResult<Vec<ShortcutEntry>> {
    let user = steam::detect::find_user(&user_steam_id)?;
    let result = steam::shortcuts::read_shortcuts(&user.shortcuts_path).map_err(Into::into);
    if let Ok(ref shortcuts) = result {
        debug!(count = shortcuts.len(), "Shortcuts loaded");
    }
    result
}

#[tauri::command]
#[instrument(skip_all, fields(user = %request.user_steam_id))]
#[specta::specta]
pub fn scan_sources(
    app: tauri::AppHandle,
    request: ScanRequest,
) -> CommandResult<Vec<ImportCandidate>> {
    let user = steam::detect::find_user(&request.user_steam_id)?;
    let settings = super::load_settings().unwrap_or_default();
    let result = crate::importers::scan::scan_sources_with_progress(
        |event| {
            if let Err(error) = event.emit(&app) {
                tracing::warn!(%error, "Could not send scan progress");
            }
        },
        &user,
        &request,
        &settings,
    )
    .map_err(Into::into);
    if let Ok(ref candidates) = result {
        info!(total = candidates.len(), "Scan complete");
    }
    result
}

#[tauri::command]
#[instrument(skip_all, fields(user = %user_steam_id, candidates = candidates.len()))]
#[specta::specta]
pub fn create_preview_plan(
    user_steam_id: String,
    candidates: Vec<ImportCandidate>,
    options: crate::models::Settings,
) -> CommandResult<PreviewPlan> {
    let user = steam::detect::find_user(&user_steam_id)?;
    let plan = steam::plan::build_preview_plan(
        &user,
        &candidates,
        &options,
        &crate::paths::new_backup_dir(),
    )?;
    info!(
        changes = plan.changes.len(),
        backups = plan.backups.len(),
        "Preview plan created"
    );
    Ok(plan)
}

#[tauri::command]
#[instrument(skip_all, fields(name = ?request.display_name, exe = %request.executable_path.display()))]
#[specta::specta]
pub fn create_manual_candidate(request: ManualImportRequest) -> CommandResult<ImportCandidate> {
    let user = steam::detect::find_user(&request.user_steam_id)?;
    let settings = super::load_settings().unwrap_or_default();
    // A name typed by the user wins over the one in Steam.
    let use_steam_name = request.display_name.is_none();
    let mut candidate = crate::importers::manual::candidate(&user, request);
    let existing_shortcuts = steam::shortcuts::read_shortcuts_or_empty(&user.shortcuts_path);
    crate::importers::scan::prepare_candidate(
        &mut candidate,
        &user,
        &existing_shortcuts,
        &settings,
        use_steam_name,
    );
    Ok(candidate)
}

#[tauri::command]
#[instrument(skip(app, request), fields(user = %request.plan.user_steam_id, candidates = request.candidates.len()))]
#[specta::specta]
pub fn apply_plan(app: tauri::AppHandle, request: ApplyRequest) -> CommandResult<ApplyResult> {
    let result = steam::apply::apply_plan_with_progress(
        |event| {
            if let Err(error) = event.emit(&app) {
                tracing::warn!(%error, "Could not send apply progress");
            }
        },
        request,
    )
    .map_err(Into::into);
    if let Ok(ref r) = result {
        info!(
            applied = r.applied_changes.len(),
            backups = r.backups_created.len(),
            "Plan applied"
        );
    }
    result
}
