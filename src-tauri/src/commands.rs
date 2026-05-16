use crate::{
    error::{io_context, AppError, CommandError},
    models::{
        ApplyRequest, ApplyResult, BackupPlan, ChangeKind, ImportCandidate, ManualImportRequest,
        Options, PlannedChange, PreviewPlan, ScanRequest, SteamInstallation, UserSettings,
    },
    steam,
};
use chrono::Utc;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;
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
pub fn read_shortcuts_for_user(
    user_steam_id: String,
) -> CommandResult<Vec<crate::models::ShortcutEntry>> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(user_steam_id.clone()))?;

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
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .iter()
        .find(|user| user.steam_id == request.user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(request.user_steam_id.clone()))?;

    let result =
        steam::sources::scan_sources_with_progress(&app, user, &request).map_err(Into::into);
    if let Ok(ref candidates) = result {
        info!(total = candidates.len(), "Scan complete");
    }
    result
}

#[tauri::command]
#[instrument(skip(app, candidates, options), fields(user = %user_steam_id, candidates = candidates.len()))]
pub fn create_preview_plan(
    app: tauri::AppHandle,
    user_steam_id: String,
    candidates: Vec<ImportCandidate>,
    options: Options,
) -> CommandResult<PreviewPlan> {
    let install = steam::detect::detect_steam()?;
    let user = install
        .users
        .into_iter()
        .find(|user| user.steam_id == user_steam_id)
        .ok_or_else(|| AppError::UserNotFound(user_steam_id.clone()))?;

    let app_data_dir = app.path().app_data_dir().map_err(|_| {
        AppError::Message("Could not resolve app data directory for backups.".to_string())
    })?;
    let backup_root = app_data_dir
        .parent()
        .unwrap_or(&app_data_dir)
        .join("Full Steam Ahead")
        .join("backups")
        .join(Utc::now().format("%Y%m%d-%H%M%S").to_string());

    let mut files = BTreeSet::<PathBuf>::new();
    files.insert(user.shortcuts_path.clone());
    files.insert(user.collections_path.clone());

    let existing_shortcuts =
        steam::shortcuts::read_shortcuts(&user.shortcuts_path).unwrap_or_default();
    let existing_collection_app_ids =
        steam::collections::existing_managed_app_ids(&user.collections_path);

    let mut changes = Vec::new();
    for candidate in &candidates {
        let (c, artwork_files) = candidate_changes(
            candidate,
            &user.shortcuts_path,
            &user.collections_path,
            &user.grid_path,
            &options,
            &existing_shortcuts,
            &existing_collection_app_ids,
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

    let plan = PreviewPlan {
        user_steam_id,
        changes,
        files_to_change: files.into_iter().collect(),
        backups,
        warnings,
        requires_steam_restart: options.stop_steam || options.restart_steam,
    };
    info!(
        changes = plan.changes.len(),
        backups = plan.backups.len(),
        warnings = plan.warnings.len(),
        "Preview plan created"
    );
    Ok(plan)
}

#[tauri::command]
#[instrument(skip_all)]
pub fn load_settings(app: tauri::AppHandle) -> CommandResult<UserSettings> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(UserSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

#[tauri::command]
#[instrument(skip_all)]
pub fn save_settings(app: tauri::AppHandle, settings: UserSettings) -> CommandResult<()> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    let raw = serde_json::to_string_pretty(&settings)
        .map_err(|_| AppError::Message("Failed to serialize settings.".to_string()))?;
    fs::write(&path, raw).map_err(io_context(&path))?;
    Ok(())
}

fn settings_path(app: &tauri::AppHandle) -> CommandResult<PathBuf> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Message("Could not resolve app data directory.".to_string()))?;
    Ok(base
        .parent()
        .unwrap_or(&base)
        .join("Full Steam Ahead")
        .join("settings.json"))
}

#[tauri::command]
#[instrument(fields(name = ?request.display_name, exe = %request.executable_path.display()))]
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
#[instrument(skip(app, request), fields(user = %request.plan.user_steam_id, candidates = request.candidates.len()))]
pub fn apply_plan(app: tauri::AppHandle, request: ApplyRequest) -> CommandResult<ApplyResult> {
    let result = steam::apply::apply_plan_with_progress(&app, request).map_err(Into::into);
    if let Ok(ref r) = result {
        info!(
            applied = r.applied_changes.len(),
            skipped = r.skipped_changes.len(),
            backups = r.backups_created.len(),
            "Plan applied"
        );
        for msg in &r.skipped_changes {
            warn!(reason = %msg, "Change skipped");
        }
    }
    result
}

fn candidate_changes(
    candidate: &ImportCandidate,
    shortcuts_path: &Path,
    collections_path: &Path,
    grid_path: &Path,
    options: &Options,
    existing_shortcuts: &[crate::models::ShortcutEntry],
    existing_collection_app_ids: &HashMap<String, HashSet<u32>>,
) -> (Vec<PlannedChange>, Vec<PathBuf>) {
    let mut changes = Vec::new();
    let mut artwork_files = Vec::new();

    let exe = candidate.effective_executable();

    let existing_shortcut = existing_shortcuts
        .iter()
        .find(|s| s.app_name.eq_ignore_ascii_case(&candidate.name));
    let shortcut_unchanged = existing_shortcut.is_some_and(|s| shortcut_is_unchanged(s, candidate));
    if !shortcut_unchanged {
        let shortcut_exists = existing_shortcut.is_some();
        changes.push(PlannedChange {
            id: format!("shortcut:{}", candidate.id),
            title: format!(
                "{} shortcut for {}",
                if shortcut_exists { "Update" } else { "Add" },
                candidate.name
            ),
            game_name: candidate.name.clone(),
            file: shortcuts_path.to_path_buf(),
            kind: if shortcut_exists {
                ChangeKind::UpdateShortcut
            } else {
                ChangeKind::AddShortcut
            },
            destructive: false,
            details: format!("Create a non-Steam shortcut from {}", exe.display()),
        });
    }

    let collection_name = candidate.source.collection_name();
    let already_in_collection = existing_collection_app_ids
        .get(&collection_name)
        .is_some_and(|ids| {
            // Check both the effective exe and the raw exe path so that games previously
            // imported without Via Launcher are still recognised after the toggle changes.
            [candidate.effective_executable(), &candidate.executable_path]
                .iter()
                .any(|p| {
                    let id =
                        steam::non_steam_app_id(&format!("\"{}\"", p.display()), &candidate.name);
                    ids.contains(&id)
                })
        });
    changes.push(PlannedChange {
        id: format!("collection:{}:{}", collection_name, candidate.id),
        title: format!("Add {} to {} collection", candidate.name, collection_name),
        game_name: candidate.name.clone(),
        file: collections_path.to_path_buf(),
        kind: ChangeKind::UpdateCollections,
        destructive: already_in_collection,
        details: "Only app-managed collections will be changed; user collections are preserved."
            .to_string(),
    });

    let app_id = steam::non_steam_app_id(&format!("\"{}\"", exe.display()), &candidate.name);
    for asset in steam::artwork::selected_artwork_assets(candidate) {
        let is_official_steam = asset.source == crate::models::ArtworkSource::OfficialSteam;

        // Skip if apply would skip it too (file exists, replace disabled, not a local override)
        if asset.will_replace_existing
            && !options.replace_existing_artwork
            && !is_official_steam
            && asset.source != crate::models::ArtworkSource::LocalFile
        {
            continue;
        }

        let file = steam::artwork::target_path(grid_path, app_id, &asset.kind, &asset.path_or_url);
        if asset.will_replace_existing {
            artwork_files.push(file.clone());
        }

        // Official Steam artwork re-downloads the same content
        if is_official_steam && asset.will_replace_existing {
            continue;
        }

        changes.push(PlannedChange {
            id: format!("artwork:{}:{}", candidate.id, asset.kind.label()),
            title: format!("Set {} artwork for {}", asset.kind.label(), candidate.name),
            game_name: candidate.name.clone(),
            file,
            kind: ChangeKind::WriteArtwork,
            destructive: asset.will_replace_existing,
            details: format!(
                "Use {} artwork from {}",
                asset.kind.label(),
                asset.source.label()
            ),
        });
    }

    (changes, artwork_files)
}

fn shortcut_is_unchanged(
    existing: &crate::models::ShortcutEntry,
    candidate: &ImportCandidate,
) -> bool {
    let exe = format!("\"{}\"", candidate.effective_executable().display());
    let start_dir = format!("\"{}\"", candidate.effective_start_dir().display());
    let launch_options = candidate.effective_launch_options().unwrap_or("");
    existing.exe == exe
        && existing.start_dir == start_dir
        && existing.launch_options == launch_options
        && existing.tags == candidate.tags
}

#[tauri::command]
#[instrument(skip_all)]
pub fn close_app(app: tauri::AppHandle) {
    info!("Application closing");
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtworkMode, ArtworkPlan, ImportSource};

    fn make_candidate(
        exe: &str,
        start_dir: &str,
        launch_options: Option<&str>,
        tags: Vec<String>,
    ) -> ImportCandidate {
        ImportCandidate {
            id: "test".to_string(),
            source: ImportSource::Manual,
            name: "Test Game".to_string(),
            executable_path: PathBuf::from(exe),
            start_dir: PathBuf::from(start_dir),
            launch_options: launch_options.map(String::from),
            existing_app_id: None,
            matched_steam_app_id: None,
            tags,
            artwork: ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing: Vec::new(),
                proposed: Vec::new(),
            },
            url_scheme: None,
            launcher_path: None,
            use_launcher_url: false,
        }
    }

    fn make_shortcut_matching(candidate: &ImportCandidate) -> crate::models::ShortcutEntry {
        crate::models::ShortcutEntry {
            app_id: 0,
            app_name: candidate.name.clone(),
            exe: format!("\"{}\"", candidate.effective_executable().display()),
            start_dir: format!("\"{}\"", candidate.effective_start_dir().display()),
            icon: String::new(),
            shortcut_path: String::new(),
            launch_options: candidate
                .effective_launch_options()
                .unwrap_or("")
                .to_string(),
            is_hidden: false,
            allow_desktop_config: true,
            allow_overlay: true,
            open_vr: false,
            devkit: false,
            devkit_game_id: String::new(),
            last_play_time: 0,
            tags: candidate.tags.clone(),
        }
    }

    #[test]
    fn shortcut_unchanged_when_all_fields_match() {
        let candidate = make_candidate("game.exe", "C:\\Games", Some("--flag"), vec!["Epic".to_string()]);
        let shortcut = make_shortcut_matching(&candidate);
        assert!(shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_changed_when_exe_differs() {
        let candidate = make_candidate("game.exe", "C:\\Games", None, vec![]);
        let mut shortcut = make_shortcut_matching(&candidate);
        shortcut.exe = "\"other.exe\"".to_string();
        assert!(!shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_changed_when_start_dir_differs() {
        let candidate = make_candidate("game.exe", "C:\\Games", None, vec![]);
        let mut shortcut = make_shortcut_matching(&candidate);
        shortcut.start_dir = "\"C:\\Other\"".to_string();
        assert!(!shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_changed_when_launch_options_differ() {
        let candidate = make_candidate("game.exe", "C:\\Games", Some("--new"), vec![]);
        let mut shortcut = make_shortcut_matching(&candidate);
        shortcut.launch_options = "--old".to_string();
        assert!(!shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_changed_when_tags_differ() {
        let candidate = make_candidate("game.exe", "C:\\Games", None, vec!["Epic".to_string()]);
        let mut shortcut = make_shortcut_matching(&candidate);
        shortcut.tags = vec!["GOG".to_string()];
        assert!(!shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_changed_when_empty_options_vs_some() {
        let candidate = make_candidate("game.exe", "C:\\Games", Some("--flag"), vec![]);
        let mut shortcut = make_shortcut_matching(&candidate);
        shortcut.launch_options = String::new();
        assert!(!shortcut_is_unchanged(&shortcut, &candidate));
    }

    #[test]
    fn shortcut_uses_launcher_path_when_use_launcher_url() {
        let mut candidate =
            make_candidate("explorer.exe", "C:\\WINDOWS", Some("shell:game"), vec![]);
        candidate.use_launcher_url = true;
        candidate.url_scheme = Some("shell:game".to_string());
        candidate.launcher_path = Some(PathBuf::from("launcher/launcher.exe"));

        // A shortcut built from the effective (launcher) path is unchanged.
        let matching = make_shortcut_matching(&candidate);
        assert!(shortcut_is_unchanged(&matching, &candidate));

        // A shortcut pointing at the raw executable_path is seen as changed.
        let mut mismatched = matching.clone();
        mismatched.exe = format!("\"{}\"", candidate.executable_path.display());
        assert!(!shortcut_is_unchanged(&mismatched, &candidate));
    }
}
