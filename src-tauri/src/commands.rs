use crate::{
    error::{AppError, CommandError},
    models::{
        ApplyOptions, ApplyRequest, ApplyResult, BackupPlan, ChangeKind, ImportCandidate,
        ManualImportRequest, PlannedChange, PreviewPlan, ScanRequest, SteamInstallation,
    },
    steam,
};
use chrono::Utc;
use std::{collections::BTreeSet, path::{Path, PathBuf}};

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
pub fn detect_steam() -> CommandResult<SteamInstallation> {
    steam::detect::detect_steam().map_err(Into::into)
}

#[tauri::command]
pub fn read_shortcuts_for_user(
    user_steam_id: String,
) -> CommandResult<Vec<crate::models::ShortcutEntry>> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(user_steam_id.clone()))?;

    steam::shortcuts::read_shortcuts(&user.shortcuts_path).map_err(Into::into)
}

#[tauri::command]
pub fn scan_sources(app: tauri::AppHandle, request: ScanRequest) -> CommandResult<Vec<ImportCandidate>> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .iter()
        .find(|user| user.steam_id == request.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.user_steam_id.clone()))?;

    steam::sources::scan_sources_with_progress(&app, user, &request).map_err(Into::into)
}

#[tauri::command]
pub fn create_preview_plan(
    user_steam_id: String,
    candidates: Vec<ImportCandidate>,
    options: ApplyOptions,
) -> CommandResult<PreviewPlan> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(user_steam_id.clone()))?;

    let backup_root = user
        .shortcuts_path
        .parent()
        .unwrap_or(&user.shortcuts_path)
        .join("full-steam-ahead-backups")
        .join(Utc::now().format("%Y%m%d-%H%M%S").to_string());

    let mut files = BTreeSet::<PathBuf>::new();
    files.insert(user.shortcuts_path.clone());
    if options.write_collections {
        files.insert(user.collections_path.clone());
    }

    let mut changes = Vec::new();
    for candidate in &candidates {
        let (c, artwork_files) = candidate_changes(
            candidate,
            &user.shortcuts_path,
            &user.collections_path,
            &user.grid_path,
            &options,
        );
        changes.extend(c);
        files.extend(artwork_files);
    }

    let backups = files
        .iter()
        .filter(|source| source.exists())
        .map(|source| BackupPlan {
            source: source.clone(),
            destination: backup_root.join(
                source
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("steam-file.backup"),
            ),
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if candidates.is_empty() {
        warnings.push(
            "No import candidates selected; applying this plan will not change shortcuts."
                .to_string(),
        );
    }
    if options.stop_steam || options.restart_steam {
        warnings.push(
            "Steam process control is opt-in. Review open downloads or cloud sync before applying."
                .to_string(),
        );
    }

    Ok(PreviewPlan {
        user_steam_id,
        changes,
        files_to_change: files.into_iter().collect(),
        backups,
        warnings,
        requires_steam_restart: options.stop_steam || options.restart_steam,
    })
}

#[tauri::command]
pub fn create_manual_candidate(request: ManualImportRequest) -> CommandResult<ImportCandidate> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .iter()
        .find(|user| user.steam_id == request.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.user_steam_id.clone()))?;

    Ok(crate::importers::manual::candidate_with_grid_path(
        request,
        &user.grid_path,
    ))
}

#[tauri::command]
pub fn apply_plan(app: tauri::AppHandle, request: ApplyRequest) -> CommandResult<ApplyResult> {
    steam::apply::apply_plan_with_progress(&app, request).map_err(Into::into)
}

fn candidate_changes(
    candidate: &ImportCandidate,
    shortcuts_path: &Path,
    collections_path: &Path,
    grid_path: &Path,
    options: &ApplyOptions,
) -> (Vec<PlannedChange>, Vec<PathBuf>) {
    let mut changes = Vec::new();
    let mut artwork_files = Vec::new();

    let exe = candidate.effective_executable();
    changes.push(PlannedChange {
        id: format!("shortcut:{}", candidate.id),
        title: format!("Add shortcut for {}", candidate.name),
        game_name: candidate.name.clone(),
        file: shortcuts_path.to_path_buf(),
        kind: ChangeKind::AddShortcut,
        destructive: false,
        details: format!("Create a non-Steam shortcut from {}", exe.display()),
    });

    if options.write_collections {
        changes.push(PlannedChange {
            id: format!(
                "collection:{}:{}",
                candidate.source.collection_name(),
                candidate.id
            ),
            title: format!(
                "Add {} to {} collection",
                candidate.name,
                candidate.source.collection_name()
            ),
            game_name: candidate.name.clone(),
            file: collections_path.to_path_buf(),
            kind: ChangeKind::UpdateCollections,
            destructive: false,
            details: "Only app-managed collections will be changed; user collections are preserved."
                .to_string(),
        });
    }

    let app_id = steam::non_steam_app_id(
        &format!("\"{}\"", exe.display()),
        &candidate.name,
    );
    for asset in steam::artwork::selected_artwork_assets(candidate) {
        let file =
            steam::artwork::target_path(grid_path, app_id, &asset.kind, &asset.path_or_url);
        artwork_files.push(file.clone());
        changes.push(PlannedChange {
            id: format!("artwork:{}:{}", candidate.id, asset.kind.label()),
            title: format!("Set {} artwork for {}", asset.kind.label(), candidate.name),
            game_name: candidate.name.clone(),
            file,
            kind: ChangeKind::WriteArtwork,
            destructive: asset.will_replace_existing && options.replace_existing_artwork,
            details: format!(
                "Use {} artwork from {}",
                asset.kind.label(),
                asset.source.label()
            ),
        });
    }

    (changes, artwork_files)
}
