use crate::{
    error::AppResult,
    models::{
        ArtworkSource, BackupPlan, ChangeKind, ImportCandidate, PlannedChange, PreviewPlan,
        Settings, ShortcutEntry, SteamUser,
    },
};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
};

pub fn build_preview_plan(
    user: &SteamUser,
    candidates: &[ImportCandidate],
    options: &Settings,
    backup_root: &Path,
) -> AppResult<PreviewPlan> {
    let mut files = BTreeSet::<PathBuf>::new();
    files.insert(user.shortcuts_path.clone());
    files.insert(user.collections_path.clone());

    let existing_shortcuts = super::shortcuts::read_shortcuts_or_empty(&user.shortcuts_path);
    let existing_collection_app_ids =
        super::collections::existing_managed_app_ids(&user.collections_path);

    let mut candidates = candidates.to_vec();
    super::shortcuts::link_existing(&mut candidates, &existing_shortcuts);
    let updated_shortcuts =
        super::shortcuts::with_candidates(&existing_shortcuts, &candidates, &user.grid_path);

    let mut changes = Vec::new();
    for candidate in &candidates {
        let (c, artwork_files) = candidate_changes(
            candidate,
            &user.shortcuts_path,
            &user.collections_path,
            &user.grid_path,
            options,
            shortcut_change(candidate, &existing_shortcuts, &updated_shortcuts),
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
    options: &Settings,
    shortcut_change: Option<ChangeKind>,
    existing_collection_app_ids: &HashMap<String, HashSet<u32>>,
) -> (Vec<PlannedChange>, Vec<PathBuf>) {
    let mut changes = Vec::new();
    let mut artwork_files = Vec::new();

    let app_id = super::candidate_app_id(candidate);

    if let Some(kind) = shortcut_change {
        changes.push(PlannedChange {
            id: format!("shortcut:{}", candidate.id),
            candidate_id: candidate.id.clone(),
            game_name: candidate.name.clone(),
            file: shortcuts_path.to_path_buf(),
            kind,
            destructive: false,
            artwork_source: None,
            artwork_kind: None,
            collection_name: None,
        });
    }

    if options.create_collections {
        let collection_name = candidate.source.collection_name();
        let already_in_collection = existing_collection_app_ids
            .get(&collection_name)
            .is_some_and(|ids| ids.contains(&app_id));
        changes.push(PlannedChange {
            id: format!("collection:{}:{}", collection_name, candidate.id),
            candidate_id: candidate.id.clone(),
            game_name: candidate.name.clone(),
            file: collections_path.to_path_buf(),
            kind: ChangeKind::UpdateCollections,
            destructive: already_in_collection,
            artwork_source: None,
            artwork_kind: None,
            collection_name: Some(collection_name),
        });
    }

    for asset in super::artwork::selected_artwork_assets(candidate) {
        let is_official_steam = asset.source == ArtworkSource::OfficialSteam;
        let is_noop_delete = asset.source == ArtworkSource::Missing && !asset.will_replace_existing;

        let file = super::artwork::target_path(grid_path, app_id, &asset.kind, &asset.path_or_url);
        if asset.will_replace_existing {
            artwork_files.push(file.clone());
        }

        // Official artwork would download the same file again. Deleting an empty slot does nothing.
        if (is_official_steam && asset.will_replace_existing) || is_noop_delete {
            continue;
        }

        changes.push(PlannedChange {
            id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
            candidate_id: candidate.id.clone(),
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

/// Compares the candidate's shortcut before and after the import.
fn shortcut_change(
    candidate: &ImportCandidate,
    before: &[ShortcutEntry],
    after: &[ShortcutEntry],
) -> Option<ChangeKind> {
    let app_id = super::candidate_app_id(candidate);
    // The icon is left out because new artwork is only downloaded when applying.
    let find = |shortcuts: &[ShortcutEntry]| {
        shortcuts
            .iter()
            .find(|s| s.app_id == app_id)
            .map(|s| ShortcutEntry {
                icon: String::new(),
                ..s.clone()
            })
    };
    match (find(before), find(after)) {
        (None, _) => Some(ChangeKind::AddShortcut),
        (existing, updated) if existing != updated => Some(ChangeKind::UpdateShortcut),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtworkMode, ArtworkPlan, ImportSource};
    use crate::steam::shortcuts::{shortcut_from_candidate, with_candidates};

    fn make_candidate(launch_options: Option<&str>) -> ImportCandidate {
        ImportCandidate {
            id: "test".to_string(),
            source: ImportSource::Manual,
            name: "Test Game".to_string(),
            original_name: "Test Game".to_string(),
            executable_path: PathBuf::from("game.exe"),
            start_dir: PathBuf::from("C:\\Games"),
            launch_options: launch_options.map(String::from),
            existing_app_id: None,
            matched_steam_app_id: None,
            tags: vec!["Epic".to_string()],
            artwork: ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing: Vec::new(),
                proposed: Vec::new(),
            },
            url_scheme: None,
            launcher_path: None,
            use_launcher_url: false,
            needs_proton: false,
        }
    }

    // Imports the candidate again over the shortcut FSA made for it, after `edit` changed it in Steam.
    fn change_after(
        candidate: &ImportCandidate,
        edit: impl FnOnce(&mut ShortcutEntry),
    ) -> Option<ChangeKind> {
        let grid = Path::new("grid");
        let mut existing = shortcut_from_candidate(candidate, grid);
        edit(&mut existing);
        let mut candidate = candidate.clone();
        candidate.existing_app_id = Some(existing.app_id);
        let before = [existing];
        let after = with_candidates(&before, std::slice::from_ref(&candidate), grid);
        shortcut_change(&candidate, &before, &after)
    }

    #[test]
    fn new_shortcut_is_an_addition() {
        let candidate = make_candidate(None);
        let after = with_candidates(&[], std::slice::from_ref(&candidate), Path::new("grid"));
        let change = shortcut_change(&candidate, &[], &after);
        assert!(matches!(change, Some(ChangeKind::AddShortcut)));
    }

    #[test]
    fn unchanged_shortcut_is_no_change() {
        let change = change_after(&make_candidate(Some("--flag")), |_| {});
        assert!(change.is_none());
    }

    #[test]
    fn renamed_shortcut_is_an_update() {
        let change = change_after(&make_candidate(None), |s| s.app_name = "Old Name".into());
        assert!(matches!(change, Some(ChangeKind::UpdateShortcut)));
    }

    #[test]
    fn other_exe_is_an_update() {
        let change = change_after(&make_candidate(None), |s| s.exe = "\"other.exe\"".into());
        assert!(matches!(change, Some(ChangeKind::UpdateShortcut)));
    }

    #[test]
    fn other_start_dir_is_an_update() {
        let change = change_after(&make_candidate(None), |s| {
            s.start_dir = "\"C:\\Other\"".into()
        });
        assert!(matches!(change, Some(ChangeKind::UpdateShortcut)));
    }

    #[test]
    fn launch_options_and_tags_changed_in_steam_are_no_change() {
        let change = change_after(&make_candidate(Some("--new")), |s| {
            s.launch_options = "--old".into();
            s.tags = vec!["Favorites".into()];
        });
        assert!(change.is_none());
    }

    #[test]
    fn shortcut_pointing_at_the_game_is_updated_to_the_launcher() {
        let mut candidate = make_candidate(Some("shell:game"));
        candidate.use_launcher_url = true;
        candidate.url_scheme = Some("shell:game".to_string());
        candidate.launcher_path = Some(PathBuf::from("launcher/launcher.exe"));

        assert!(change_after(&candidate, |_| {}).is_none());
        let change = change_after(&candidate, |s| {
            s.exe = crate::importers::quote_path(&candidate.executable_path)
        });
        assert!(matches!(change, Some(ChangeKind::UpdateShortcut)));
    }
}
