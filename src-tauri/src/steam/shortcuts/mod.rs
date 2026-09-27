mod vdf;

pub use vdf::{read_shortcuts, write_shortcuts};

use crate::{
    importers::quote_path,
    models::{ArtworkKind, ArtworkSource, ImportCandidate, ShortcutEntry},
    steam::{artwork, candidate_app_id, non_steam_app_id},
};
use std::path::Path;

/// Finds the shortcut a candidate was imported as by its app id, which Steam keeps on a rename.
pub fn find_existing<'a>(
    shortcuts: &'a [ShortcutEntry],
    candidate: &ImportCandidate,
) -> Option<&'a ShortcutEntry> {
    // Both paths so games are still recognised after the launcher toggle changes.
    let computed_ids = [
        Some(candidate.executable_path.as_path()),
        candidate.launcher_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(|exe| non_steam_app_id(&quote_path(exe), &candidate.name));

    candidate
        .existing_app_id
        .into_iter()
        .chain(computed_ids)
        .find_map(|id| shortcuts.iter().find(|s| s.app_id == id))
}

/// Points each candidate at the shortcut it already has, or at none.
pub fn link_existing(candidates: &mut [ImportCandidate], shortcuts: &[ShortcutEntry]) {
    for candidate in candidates {
        candidate.existing_app_id = find_existing(shortcuts, candidate).map(|s| s.app_id);
    }
}

/// The shortcuts after importing the candidates. Applying writes these and the preview compares them.
pub fn with_candidates(
    existing: &[ShortcutEntry],
    candidates: &[ImportCandidate],
    grid_path: &Path,
) -> Vec<ShortcutEntry> {
    let mut shortcuts = existing.to_vec();
    for candidate in candidates {
        let shortcut = shortcut_from_candidate(candidate, grid_path);
        upsert(&mut shortcuts, shortcut, artwork::changes_icon(candidate));
    }
    shortcuts
}

/// Adds the shortcut, or updates the fields FSA manages on the one with the same app id.
/// The icon, tags and launch options changed in Steam are kept.
pub fn upsert(existing: &mut Vec<ShortcutEntry>, mut shortcut: ShortcutEntry, replace_icon: bool) {
    if shortcut.app_id == 0 {
        shortcut.app_id = non_steam_app_id(&shortcut.exe, &shortcut.app_name);
    }

    let Some(item) = existing
        .iter_mut()
        .find(|item| item.app_id == shortcut.app_id)
    else {
        existing.push(shortcut);
        return;
    };

    item.app_name = shortcut.app_name;
    item.start_dir = shortcut.start_dir;
    // Launch options only work with the exe they were written for.
    if item.exe != shortcut.exe {
        item.exe = shortcut.exe;
        item.launch_options = shortcut.launch_options;
    }
    if replace_icon {
        item.icon = shortcut.icon;
    }
}

/// Reads the shortcuts for a scan or preview. Applying reads them again and stops on errors.
pub fn read_shortcuts_or_empty(path: &Path) -> Vec<ShortcutEntry> {
    read_shortcuts(path).unwrap_or_else(|error| {
        tracing::warn!(%error, "Existing shortcuts could not be read");
        Vec::new()
    })
}

pub fn shortcut_from_candidate(candidate: &ImportCandidate, grid_path: &Path) -> ShortcutEntry {
    let exe = candidate.effective_executable();
    ShortcutEntry {
        app_id: candidate_app_id(candidate),
        app_name: candidate.name.clone(),
        exe: quote_path(exe),
        start_dir: quote_path(candidate.effective_start_dir()),
        icon: shortcut_icon(candidate, grid_path),
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

fn shortcut_icon(candidate: &ImportCandidate, grid_path: &Path) -> String {
    let fallback = candidate.executable_path.display().to_string();
    let Some(asset) = artwork::selected_artwork_assets(candidate)
        .into_iter()
        .find(|asset| asset.kind == ArtworkKind::Icon)
    else {
        return fallback;
    };

    let icon_path = match asset.source {
        ArtworkSource::ExistingCustom | ArtworkSource::Missing => {
            Path::new(&asset.path_or_url).to_path_buf()
        }
        ArtworkSource::OfficialSteam | ArtworkSource::SteamGridDb | ArtworkSource::LocalFile => {
            let app_id = candidate_app_id(candidate);
            artwork::target_path(grid_path, app_id, &ArtworkKind::Icon, &asset.path_or_url)
        }
    };

    if icon_path.exists() {
        icon_path.display().to_string()
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtworkMode, ArtworkPlan, ImportSource};
    use std::path::PathBuf;

    fn make_shortcut(name: &str, exe: &str) -> ShortcutEntry {
        ShortcutEntry {
            app_id: 12345,
            app_name: name.to_string(),
            exe: exe.to_string(),
            start_dir: "\"C:\\Games\"".to_string(),
            icon: String::new(),
            shortcut_path: String::new(),
            launch_options: String::new(),
            is_hidden: false,
            allow_desktop_config: true,
            allow_overlay: true,
            open_vr: false,
            devkit: false,
            devkit_game_id: String::new(),
            last_play_time: 0,
            tags: Vec::new(),
        }
    }

    #[test]
    fn upsert_adds_new_shortcut() {
        let mut existing = Vec::new();
        upsert(
            &mut existing,
            make_shortcut("New Game", "\"new.exe\""),
            true,
        );
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].app_name, "New Game");
    }

    #[test]
    fn upsert_adds_when_only_the_name_matches() {
        let mut existing = vec![{
            let mut s = make_shortcut("My Game", "\"old.exe\"");
            s.app_id = 111;
            s
        }];
        let other = {
            let mut s = make_shortcut("My Game", "\"new.exe\"");
            s.app_id = 222;
            s
        };
        upsert(&mut existing, other, true);
        assert_eq!(existing.len(), 2);
    }

    #[test]
    fn upsert_renames_by_app_id_and_keeps_user_state() {
        let mut existing = vec![{
            let mut s = make_shortcut("Old Name", "\"game.exe\"");
            s.is_hidden = true;
            s.last_play_time = 1_700_000_000;
            s.allow_overlay = false;
            s.icon = "custom.ico".to_string();
            s.launch_options = "-dx11".to_string();
            s.tags = vec!["Favorites".to_string()];
            s
        }];
        upsert(
            &mut existing,
            make_shortcut("New Name", "\"game.exe\""),
            false,
        );
        assert_eq!(existing.len(), 1, "must not add a duplicate");
        assert_eq!(existing[0].app_name, "New Name");
        assert!(existing[0].is_hidden);
        assert_eq!(existing[0].last_play_time, 1_700_000_000);
        assert!(!existing[0].allow_overlay);
        assert_eq!(existing[0].icon, "custom.ico");
        assert_eq!(existing[0].launch_options, "-dx11");
        assert_eq!(existing[0].tags, ["Favorites"]);
    }

    #[test]
    fn upsert_replaces_launch_options_along_with_the_exe() {
        let mut existing = vec![{
            let mut s = make_shortcut("Game", "\"launcher.exe\"");
            s.launch_options = "launch game".to_string();
            s
        }];
        upsert(&mut existing, make_shortcut("Game", "\"game.exe\""), false);
        assert_eq!(existing[0].exe, "\"game.exe\"");
        assert_eq!(existing[0].launch_options, "");
    }

    #[test]
    fn upsert_computes_app_id_when_zero() {
        let mut existing = Vec::new();
        let mut s = make_shortcut("Auto ID", "\"auto.exe\"");
        s.app_id = 0;
        upsert(&mut existing, s, true);
        assert_ne!(existing[0].app_id, 0, "app_id must be computed");
        assert!(
            existing[0].app_id & 0x8000_0000 != 0,
            "computed app_id must have high bit set"
        );
    }

    fn make_candidate(name: &str, exe: &str, launch_options: Option<&str>) -> ImportCandidate {
        ImportCandidate {
            id: "test".to_string(),
            source: ImportSource::Manual,
            name: name.to_string(),
            original_name: name.to_string(),
            executable_path: PathBuf::from(exe),
            start_dir: PathBuf::from("C:\\Games"),
            launch_options: launch_options.map(String::from),
            existing_app_id: None,
            matched_steam_app_id: None,
            tags: Vec::new(),
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

    // A shortcut as FSA wrote it for the candidate, then renamed in Steam.
    fn renamed_in_steam(candidate: &ImportCandidate, new_name: &str) -> ShortcutEntry {
        let exe = quote_path(&candidate.executable_path);
        ShortcutEntry {
            app_id: non_steam_app_id(&exe, &candidate.name),
            app_name: new_name.to_string(),
            exe,
            launch_options: candidate.launch_options.clone().unwrap_or_default(),
            ..ShortcutEntry::default()
        }
    }

    #[test]
    fn finds_shortcut_renamed_in_steam_by_app_id() {
        let candidate = make_candidate("Game", "launcher.exe", Some("launch a"));
        let other = make_candidate("Other", "launcher.exe", Some("launch b"));
        let shortcuts = [
            renamed_in_steam(&other, "Other"),
            renamed_in_steam(&candidate, "My Renamed Game"),
        ];
        let found = find_existing(&shortcuts, &candidate).unwrap();
        assert_eq!(found.app_name, "My Renamed Game");
    }

    #[test]
    fn finds_shortcut_after_the_launcher_toggle_changed() {
        let mut candidate = make_candidate("Game", "game.exe", None);
        candidate.launcher_path = Some(PathBuf::from("launcher.exe"));
        let exe = quote_path(Path::new("launcher.exe"));
        let via_launcher = ShortcutEntry {
            app_id: non_steam_app_id(&exe, "Game"),
            exe,
            ..ShortcutEntry::default()
        };
        assert!(find_existing(&[via_launcher], &candidate).is_some());
    }

    #[test]
    fn ignores_a_shortcut_with_only_the_same_name() {
        let candidate = make_candidate("Game", "game.exe", None);
        let mut shortcut = renamed_in_steam(&candidate, "Game");
        shortcut.app_id = 42;
        assert!(find_existing(&[shortcut], &candidate).is_none());
    }

    #[test]
    fn linked_app_id_survives_a_rename_in_fsa() {
        let mut candidate = make_candidate("Game", "game.exe", None);
        let shortcut = renamed_in_steam(&candidate, "Game");
        candidate.existing_app_id = Some(shortcut.app_id);
        candidate.name = "Renamed In FSA".to_string();
        candidate.executable_path = PathBuf::from("moved/game.exe");
        assert!(find_existing(&[shortcut], &candidate).is_some());
    }

    #[test]
    fn link_clears_a_stale_app_id() {
        let mut candidates = [make_candidate("Game", "game.exe", None)];
        candidates[0].existing_app_id = Some(7);
        link_existing(&mut candidates, &[]);
        assert_eq!(candidates[0].existing_app_id, None);
    }
}
