use crate::{
    importers::quote_path,
    models::{ImportCandidate, ShortcutEntry},
};

/// A title is presentation, not identity. Keep Steam's stored ID after user edits,
/// and also recognise older/manual imports by their actual launch target.
pub(super) fn existing_shortcut<'a>(
    candidate: &ImportCandidate,
    existing: &'a [ShortcutEntry],
) -> Option<&'a ShortcutEntry> {
    existing.iter().find(|shortcut| {
        let paths = [
            Some(candidate.executable_path.as_path()),
            candidate.launcher_path.as_deref(),
        ];
        let id_matches = shortcut.app_id != 0
            && (candidate.existing_app_id == Some(shortcut.app_id)
                || paths.into_iter().flatten().any(|path| {
                    super::non_steam_app_id(&quote_path(path), &candidate.name) == shortcut.app_id
                }));
        if id_matches {
            return true;
        }

        let options = [
            candidate.launch_options.as_deref(),
            candidate.url_scheme.as_deref(),
        ];
        let target_matches = options.into_iter().flatten().any(|options| {
            launch_identity(options).is_some_and(|identity| {
                launch_identity(&shortcut.launch_options).as_ref() == Some(&identity)
                    || launch_identity(&shortcut.exe).as_ref() == Some(&identity)
            })
        });
        if target_matches {
            return true;
        }

        // Unknown launchers and shared executables must match the whole command.
        paths.into_iter().flatten().any(|path| {
            same_path(&shortcut.exe, &path.to_string_lossy())
                && shortcut.launch_options.trim()
                    == candidate.effective_launch_options().unwrap_or("").trim()
                && same_path(
                    &shortcut.start_dir,
                    &candidate.effective_start_dir().to_string_lossy(),
                )
        })
    })
}

fn same_path(left: &str, right: &str) -> bool {
    let left = left.trim().trim_matches('"');
    let right = right.trim().trim_matches('"');
    #[cfg(windows)]
    return left
        .replace('\\', "/")
        .eq_ignore_ascii_case(&right.replace('\\', "/"));
    #[cfg(not(windows))]
    return left == right;
}

fn launch_identity(command: &str) -> Option<String> {
    // Only a leading, complete launch argument is considered, never a substring
    // inside an arbitrary wrapper command or a similarly numbered game ID.
    let argument = command.split_whitespace().next()?.trim_matches(['"', '\'']);
    if let Some(app) = argument.strip_prefix("shell:AppsFolder\\") {
        return app.contains('!').then(|| format!("appx:{app}"));
    }
    let url = reqwest::Url::parse(argument).ok()?;
    match (url.scheme(), url.host_str()) {
        ("uplay", Some("launch")) => {
            let mut segments = url.path_segments()?;
            let id = segments.next()?;
            if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Some(format!("uplay:{id}"))
        }
        ("com.epicgames.launcher", Some("apps")) if url.path().len() > 1 => {
            Some(format!("epic:{}", url.path()))
        }
        _ => None,
    }
}

pub(super) fn mark_existing(candidates: &mut [ImportCandidate], existing: &[ShortcutEntry]) {
    for candidate in candidates {
        candidate.existing_app_id = existing_shortcut(candidate, existing).map(|s| s.app_id);
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::models::{ArtworkMode, ArtworkPlan, ImportSource};
    use std::path::PathBuf;

    pub(crate) fn avatar_candidate() -> ImportCandidate {
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
                proposed: vec![],
            },
            url_scheme: Some("uplay://launch/4740/0".into()),
            launcher_path: None,
            use_launcher_url: false,
            needs_proton: false,
        }
    }

    #[test]
    fn fsa_game_pass_shortcut_is_recognised_after_import() {
        let mut candidate = avatar_candidate();
        candidate.name = "Call of Duty Modern Warfare 3".into();
        candidate.source = ImportSource::GamePass;
        candidate.executable_path = PathBuf::from("C:/Windows/explorer.exe");
        candidate.start_dir = PathBuf::from("C:/Windows");
        candidate.launch_options =
            Some("shell:AppsFolder\\38985CA0.MWIIIGame_5bkah9njm3e9g!codShip".into());
        candidate.url_scheme = candidate.launch_options.clone();
        let mut shortcut = super::super::sources::shortcut_from_candidate(
            &candidate,
            std::path::Path::new("grid"),
        );
        assert_eq!(
            shortcut.launch_options,
            candidate.launch_options.as_deref().unwrap()
        );
        assert!(shortcut.exe.contains("explorer.exe"));
        mark_existing(
            std::slice::from_mut(&mut candidate),
            std::slice::from_ref(&shortcut),
        );
        assert_eq!(candidate.existing_app_id, Some(shortcut.app_id));

        candidate.existing_app_id = None;
        shortcut.app_name = "My MW3".into();
        assert!(existing_shortcut(&candidate, &[shortcut]).is_some());
    }

    #[test]
    fn renamed_avatar_matches_its_launcher_id() {
        let candidate = avatar_candidate();
        let shortcut = ShortcutEntry {
            app_id: 123,
            app_name: "Avatar: Frontiers of Pandora".into(),
            exe: "\"D:/Launchers/UbisoftConnect.exe\"".into(),
            launch_options: "\"uplay://launch/4740/0\" -custom".into(),
            icon: "custom-avatar.ico".into(),
            tags: vec!["Couch games".into()],
            ..Default::default()
        };
        assert!(existing_shortcut(&candidate, &[shortcut]).is_some());
    }

    #[test]
    fn stored_id_survives_renaming_and_custom_launch_options() {
        let candidate = avatar_candidate();
        let shortcut = ShortcutEntry {
            app_id: super::super::non_steam_app_id(&quote_path(&candidate.executable_path), "AFOP"),
            app_name: "Avatar: Frontiers of Pandora".into(),
            exe: "custom-wrapper.cmd".into(),
            launch_options: "--custom".into(),
            ..Default::default()
        };
        assert!(existing_shortcut(&candidate, &[shortcut]).is_some());
    }

    #[test]
    fn same_title_or_shared_launcher_is_not_identity() {
        let candidate = avatar_candidate();
        for options in ["uplay://launch/47400/0", "uplay://launch/99/0", ""] {
            let shortcut = ShortcutEntry {
                app_name: "AFOP".into(),
                exe: "UbisoftConnect.exe".into(),
                launch_options: options.into(),
                ..Default::default()
            };
            assert!(existing_shortcut(&candidate, &[shortcut]).is_none());
        }
    }

    #[test]
    fn appx_and_epic_targets_are_name_independent() {
        for (left, right, equal) in [
            (
                "shell:AppsFolder\\Package_abc!Game",
                "shell:AppsFolder\\Package_abc!Game",
                true,
            ),
            (
                "shell:AppsFolder\\Package_abc!Game",
                "shell:AppsFolder\\Package_abc!Helper",
                false,
            ),
            (
                "com.epicgames.launcher://apps/ns%3Aitem%3Agame?action=launch",
                "com.epicgames.launcher://apps/ns%3Aitem%3Agame?silent=true",
                true,
            ),
            ("uplay://launch/4740/0", "uplay://launch/47400/0", false),
        ] {
            assert_eq!(launch_identity(left) == launch_identity(right), equal);
        }
        assert!(launch_identity("cmd /c uplay://launch/4740/0").is_none());
        assert!(launch_identity("uplay://launch/").is_none());
    }

    #[test]
    fn exact_commands_recognise_renamed_manual_games() {
        let mut candidate = avatar_candidate();
        candidate.executable_path = PathBuf::from("game.exe");
        candidate.launch_options = Some("--play".into());
        candidate.url_scheme = None;
        let mut shortcut = ShortcutEntry {
            exe: "\"game.exe\"".into(),
            start_dir: "\".\"".into(),
            launch_options: "--play".into(),
            ..Default::default()
        };
        assert!(existing_shortcut(&candidate, &[shortcut.clone()]).is_some());
        shortcut.launch_options = "--different-game".into();
        assert!(existing_shortcut(&candidate, &[shortcut]).is_none());
    }
}
