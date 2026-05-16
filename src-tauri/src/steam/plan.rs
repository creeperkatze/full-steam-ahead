use crate::{
    error::AppResult,
    models::{
        ArtworkSource, BackupPlan, ChangeKind, ImportCandidate, Options, PlannedChange,
        PreviewPlan, ShortcutEntry, SteamUser,
    },
};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
};

pub fn build_preview_plan(
    user: &SteamUser,
    candidates: &[ImportCandidate],
    options: &Options,
    backup_root: &Path,
) -> AppResult<PreviewPlan> {
    let mut files = BTreeSet::<PathBuf>::new();
    files.insert(user.shortcuts_path.clone());
    files.insert(user.collections_path.clone());

    let existing_shortcuts =
        super::shortcuts::read_shortcuts(&user.shortcuts_path).unwrap_or_default();
    let existing_collection_app_ids =
        super::collections::existing_managed_app_ids(&user.collections_path);

    let mut changes = Vec::new();
    for candidate in candidates {
        let (c, artwork_files) = candidate_changes(
            candidate,
            &user.shortcuts_path,
            &user.collections_path,
            &user.grid_path,
            options,
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

    Ok(PreviewPlan {
        user_steam_id: user.steam_id.clone(),
        changes,
        files_to_change: files.into_iter().collect(),
        backups,
        requires_steam_restart: options.stop_steam || options.restart_steam,
    })
}

fn candidate_changes(
    candidate: &ImportCandidate,
    shortcuts_path: &Path,
    collections_path: &Path,
    grid_path: &Path,
    options: &Options,
    existing_shortcuts: &[ShortcutEntry],
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
            game_name: candidate.name.clone(),
            file: shortcuts_path.to_path_buf(),
            kind: if shortcut_exists {
                ChangeKind::UpdateShortcut
            } else {
                ChangeKind::AddShortcut
            },
            destructive: false,
            artwork_source: None,
            artwork_kind: None,
            collection_name: None,
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
                        super::non_steam_app_id(&format!("\"{}\"", p.display()), &candidate.name);
                    ids.contains(&id)
                })
        });
    changes.push(PlannedChange {
        id: format!("collection:{}:{}", collection_name, candidate.id),
        game_name: candidate.name.clone(),
        file: collections_path.to_path_buf(),
        kind: ChangeKind::UpdateCollections,
        destructive: already_in_collection,
        artwork_source: None,
        artwork_kind: None,
        collection_name: Some(collection_name),
    });

    let app_id = super::non_steam_app_id(&format!("\"{}\"", exe.display()), &candidate.name);
    for asset in super::artwork::selected_artwork_assets(candidate) {
        let is_official_steam = asset.source == ArtworkSource::OfficialSteam;

        // Skip if apply would skip it too (file exists, replace disabled, not a local override)
        if asset.will_replace_existing
            && !options.replace_existing_artwork
            && !is_official_steam
            && asset.source != ArtworkSource::LocalFile
        {
            continue;
        }

        let file = super::artwork::target_path(grid_path, app_id, &asset.kind, &asset.path_or_url);
        if asset.will_replace_existing {
            artwork_files.push(file.clone());
        }

        // Official Steam artwork re-downloads the same content
        if is_official_steam && asset.will_replace_existing {
            continue;
        }

        changes.push(PlannedChange {
            id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
            game_name: candidate.name.clone(),
            file,
            kind: ChangeKind::WriteArtwork,
            destructive: asset.will_replace_existing,
            artwork_source: Some(asset.source.clone()),
            artwork_kind: Some(asset.kind.clone()),
            collection_name: None,
        });
    }

    (changes, artwork_files)
}

fn shortcut_is_unchanged(existing: &ShortcutEntry, candidate: &ImportCandidate) -> bool {
    let exe = format!("\"{}\"", candidate.effective_executable().display());
    let start_dir = format!("\"{}\"", candidate.effective_start_dir().display());
    let launch_options = candidate.effective_launch_options().unwrap_or("");
    existing.exe == exe
        && existing.start_dir == start_dir
        && existing.launch_options == launch_options
        && existing.tags == candidate.tags
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

    fn make_shortcut_matching(candidate: &ImportCandidate) -> ShortcutEntry {
        ShortcutEntry {
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
        let candidate = make_candidate(
            "game.exe",
            "C:\\Games",
            Some("--flag"),
            vec!["Epic".to_string()],
        );
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
