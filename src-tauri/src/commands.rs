use crate::{
    error::{AppError, CommandError},
    models::{
        ApplyOptions, ApplyRequest, ApplyResult, BackupPlan, ChangeKind, ImportCandidate,
        ManualImportRequest, PlannedChange, PreviewPlan, ScanRequest, SteamInstallation,
    },
    steam,
};
use chrono::Utc;
use std::{collections::BTreeSet, path::PathBuf};

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
pub fn scan_sources(request: ScanRequest) -> CommandResult<Vec<ImportCandidate>> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .iter()
        .find(|user| user.steam_id == request.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.user_steam_id.clone()))?;

    steam::sources::scan_sources(user, &request).map_err(Into::into)
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
        changes.push(PlannedChange {
            id: format!("shortcut:{}", candidate.id),
            title: format!("Add shortcut for {}", candidate.name),
            file: user.shortcuts_path.clone(),
            kind: ChangeKind::AddShortcut,
            destructive: false,
            details: format!(
                "Create a non-Steam shortcut from {}",
                candidate.executable_path.display()
            ),
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
                file: user.collections_path.clone(),
                kind: ChangeKind::UpdateCollections,
                destructive: false,
                details:
                    "Only app-managed collections will be changed; user collections are preserved."
                        .to_string(),
            });
        }

        let shortcut_app_id = steam::non_steam_app_id(
            &format!("\"{}\"", candidate.executable_path.display()),
            &candidate.name,
        );
        for asset in steam::artwork::selected_artwork_assets(candidate) {
            let file = steam::artwork::target_path(
                &user.grid_path,
                shortcut_app_id,
                &asset.kind,
                &asset.path_or_url,
            );
            files.insert(file.clone());
            changes.push(PlannedChange {
                id: format!("artwork:{}:{:?}", candidate.id, asset.kind),
                title: format!("Set {:?} artwork for {}", asset.kind, candidate.name),
                file,
                kind: ChangeKind::WriteArtwork,
                destructive: asset.will_replace_existing && options.replace_existing_artwork,
                details: format!("Use {:?} artwork from {:?}", asset.kind, asset.source),
            });
        }
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
pub fn apply_plan(request: ApplyRequest) -> CommandResult<ApplyResult> {
    steam::apply::apply_plan(request).map_err(Into::into)
}
