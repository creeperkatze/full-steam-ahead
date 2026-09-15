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

    let mut existing_shortcuts = super::shortcuts::read_shortcuts(&user.shortcuts_path)?;
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
        if c.iter()
            .any(|change| matches!(change.kind, ChangeKind::AddShortcut))
        {
            existing_shortcuts.push(super::sources::shortcut_from_candidate(
                candidate,
                &user.grid_path,
            ));
        }
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
    existing_shortcuts: &[ShortcutEntry],
    existing_collection_app_ids: &HashMap<String, HashSet<u32>>,
) -> (Vec<PlannedChange>, Vec<PathBuf>) {
    let mut changes = Vec::new();
    let mut artwork_files = Vec::new();

    if super::matching::existing_shortcut(candidate, existing_shortcuts).is_some() {
        return (changes, artwork_files);
    }

    let exe = candidate.effective_executable();

    changes.push(PlannedChange {
        id: format!("shortcut:{}", candidate.id),
        game_name: candidate.name.clone(),
        file: shortcuts_path.to_path_buf(),
        kind: ChangeKind::AddShortcut,
        destructive: false,
        artwork_source: None,
        artwork_kind: None,
        collection_name: None,
    });

    if options.create_collections {
        let collection_name = candidate.source.collection_name();
        let already_in_collection = existing_collection_app_ids
            .get(&collection_name)
            .is_some_and(|ids| {
                // Checks both the effective and raw exe path so games are still recognised after the launcher toggle changes.
                [candidate.effective_executable(), &candidate.executable_path]
                    .iter()
                    .any(|p| {
                        let id = super::non_steam_app_id(
                            &format!("\"{}\"", p.display()),
                            &candidate.name,
                        );
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
    }

    let app_id = super::non_steam_app_id(&format!("\"{}\"", exe.display()), &candidate.name);
    for asset in super::artwork::selected_artwork_assets(candidate) {
        let is_official_steam = asset.source == ArtworkSource::OfficialSteam;
        let is_noop_delete = asset.source == ArtworkSource::Missing && !asset.will_replace_existing;

        let file = super::artwork::target_path(grid_path, app_id, &asset.kind, &asset.path_or_url);
        if asset.will_replace_existing {
            artwork_files.push(file.clone());
        }

        // Official Steam artwork re-downloads the same content; deleting an empty slot is a no-op.
        if (is_official_steam && asset.will_replace_existing) || is_noop_delete {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtworkAsset, ArtworkKind, ArtworkMode, ArtworkPlan, ImportSource};

    fn candidate() -> ImportCandidate {
        ImportCandidate {
            id: "ubisoft-avatar".into(),
            source: ImportSource::UbisoftConnect,
            name: "AFOP".into(),
            executable_path: PathBuf::from("UbisoftConnect.exe"),
            start_dir: PathBuf::from("."),
            launch_options: Some("uplay://launch/4740/0".into()),
            existing_app_id: None,
            matched_steam_app_id: None,
            tags: vec!["Ubisoft Connect".into()],
            artwork: ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing: vec![],
                proposed: vec![ArtworkAsset {
                    kind: ArtworkKind::Icon,
                    path_or_url: "replacement.png".into(),
                    source: ArtworkSource::LocalFile,
                    will_replace_existing: true,
                }],
            },
            url_scheme: None,
            launcher_path: None,
            use_launcher_url: false,
            needs_proton: false,
        }
    }

    fn preview(
        candidate: &ImportCandidate,
        shortcuts: &[ShortcutEntry],
    ) -> (Vec<PlannedChange>, Vec<PathBuf>) {
        candidate_changes(
            candidate,
            Path::new("shortcuts.vdf"),
            Path::new("collections"),
            Path::new("grid"),
            &Settings::default(),
            shortcuts,
            &HashMap::new(),
        )
    }

    #[test]
    fn renamed_avatar_has_no_shortcut_artwork_or_collection_changes() {
        let shortcut = ShortcutEntry {
            app_id: 123,
            app_name: "Avatar: Frontiers of Pandora".into(),
            exe: "UbisoftConnect.exe".into(),
            launch_options: "uplay://launch/4740/0 -custom".into(),
            icon: "custom.ico".into(),
            tags: vec!["Couch games".into()],
            ..Default::default()
        };
        let (changes, files) = preview(&candidate(), &[shortcut]);
        assert!(changes.is_empty());
        assert!(files.is_empty());
    }

    #[test]
    fn same_title_different_game_is_an_add_not_an_update() {
        let shortcut = ShortcutEntry {
            app_name: "AFOP".into(),
            exe: "other.exe".into(),
            launch_options: "uplay://launch/99/0".into(),
            ..Default::default()
        };
        let (changes, _) = preview(&candidate(), &[shortcut]);
        assert!(changes
            .iter()
            .any(|c| matches!(c.kind, ChangeKind::AddShortcut)));
        assert!(!changes
            .iter()
            .any(|c| matches!(c.kind, ChangeKind::UpdateShortcut)));
    }

    #[test]
    fn new_game_still_gets_shortcut_and_artwork() {
        let (changes, _) = preview(&candidate(), &[]);
        assert!(changes
            .iter()
            .any(|c| matches!(c.kind, ChangeKind::AddShortcut)));
        assert!(changes
            .iter()
            .any(|c| matches!(c.kind, ChangeKind::WriteArtwork)));
    }
}
