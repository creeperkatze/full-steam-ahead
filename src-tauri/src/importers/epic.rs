use crate::{
    error::AppResult,
    importers::{candidate_from_parts, launcher_candidate, launcher_url_pair},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

mod legacy;
#[cfg(windows)]
mod new;

#[derive(Debug, Clone)]
struct EpicLauncher {
    path: PathBuf,
    #[cfg_attr(not(unix), allow(dead_code))]
    compat_folder: Option<PathBuf>,
}

/// The games one launcher has installed.
#[derive(Debug, Clone)]
struct EpicLibrary {
    launcher: EpicLauncher,
    games: Vec<EpicGame>,
}

#[derive(Debug, Clone)]
struct EpicGame {
    display_name: String,
    install_location: String,
    launch_executable: String,
    app_name: String,
    catalog_namespace: String,
    catalog_item_id: String,
    is_managed: bool,
    requires_auth: bool,
    has_expected_dlc: bool,
}

impl EpicGame {
    fn launch_url(&self) -> String {
        format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            self.catalog_namespace, self.catalog_item_id, self.app_name
        )
    }

    fn dedupe_key(&self) -> String {
        format!(
            "{}-{}-{}",
            self.install_location, self.launch_executable, self.is_managed
        )
    }

    /// Games with no executable of their own to start.
    fn requires_launcher(&self) -> bool {
        self.is_managed || self.launch_executable.is_empty()
    }

    /// Adds titles the launcher should handle by default: DLC it assembles, and
    /// titles wanting auth arguments that only it passes.
    fn prefers_launcher(&self) -> bool {
        self.requires_launcher() || self.requires_auth || self.has_expected_dlc
    }

    fn executable_path(&self) -> PathBuf {
        Path::new(&self.install_location).join(&self.launch_executable)
    }
}

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    // The custom path can point to the data folder of either launcher.
    #[cfg(windows)]
    let (new_custom_path, custom_path) = match custom_path {
        Some(path) if new::is_data_folder(path) => (Some(path), None),
        _ => (None, custom_path),
    };

    // The new launcher comes first, so it keeps the games both launchers know.
    let mut libraries = Vec::new();
    #[cfg(windows)]
    libraries.extend(new::scan(new_custom_path));
    match legacy::scan(custom_path) {
        Ok(library) => libraries.extend(library),
        Err(error) => tracing::warn!(%error, "Could not read Epic Games Launcher manifests"),
    }

    let mut games = BTreeMap::<String, (EpicLauncher, EpicGame)>::new();
    for library in libraries {
        for game in library.games {
            games
                .entry(game.dedupe_key())
                .or_insert_with(|| (library.launcher.clone(), game));
        }
    }

    Ok(games
        .into_values()
        .map(|(launcher, game)| candidate_from_game(user, &launcher, game))
        .collect())
}

fn candidate_from_game(
    user: &SteamUser,
    launcher: &EpicLauncher,
    game: EpicGame,
) -> ImportCandidate {
    let requires_launcher = game.requires_launcher();
    let prefers_launcher = game.prefers_launcher();
    let launch_url = game.launch_url();
    let exe = game.executable_path();
    let name = game.display_name;
    let mut tags = vec!["Epic".to_string()];
    if prefers_launcher {
        tags.push("Epic Launcher".to_string());
    }

    // On Linux with Proton, embed the compat path into the launch options
    #[cfg(all(unix, not(target_os = "macos")))]
    let launch_url = if let Some(ref compat) = launcher.compat_folder {
        super::wine::proton_launch_options(compat, &launch_url)
    } else {
        launch_url
    };

    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(unused_mut))]
    let mut candidate = if requires_launcher {
        launcher_candidate(
            user,
            ImportSource::Epic,
            "epic",
            name,
            launcher.path.clone(),
            launch_url,
            tags,
        )
    } else {
        // Both launch routes are kept so the user can switch later.
        let start_dir = exe.parent().map(PathBuf::from).unwrap_or_default();
        let mut candidate = candidate_from_parts(
            user,
            ImportSource::Epic,
            "epic",
            name,
            exe,
            start_dir,
            None,
            tags,
        );
        let (launcher_path, launch_url) = launcher_url_pair(launcher.path.clone(), launch_url);
        candidate.url_scheme = Some(launch_url);
        candidate.launcher_path = Some(launcher_path);
        candidate.use_launcher_url = prefers_launcher;
        candidate
    };

    // Found inside an existing Proton prefix, so Steam must be told to launch it via Proton
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        candidate.needs_proton = launcher.compat_folder.is_some();
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> EpicGame {
        EpicGame {
            display_name: "Test Game".to_string(),
            install_location: "/games/test".to_string(),
            launch_executable: "Binaries/Win64/game.exe".to_string(),
            app_name: "testgame123".to_string(),
            catalog_namespace: "ns123".to_string(),
            catalog_item_id: "item456".to_string(),
            is_managed: false,
            requires_auth: false,
            has_expected_dlc: false,
        }
    }

    #[test]
    fn needs_launcher_when_managed() {
        let mut g = test_game();
        g.is_managed = true;
        assert!(g.requires_launcher());
        assert!(g.prefers_launcher());
    }

    #[test]
    fn needs_launcher_when_has_dlc() {
        let mut g = test_game();
        g.has_expected_dlc = true;
        assert!(g.prefers_launcher());
        // Has an executable of its own, so the user can still switch to it.
        assert!(!g.requires_launcher());
    }

    #[test]
    fn requires_launcher_without_an_executable() {
        let mut g = test_game();
        g.launch_executable = String::new();
        assert!(g.requires_launcher());
    }

    #[test]
    fn no_launcher_for_plain_game() {
        assert!(!test_game().requires_launcher());
        assert!(!test_game().prefers_launcher());
    }

    #[test]
    fn auth_prefers_the_launcher_but_keeps_the_executable() {
        let mut g = test_game();
        g.requires_auth = true;
        assert!(g.prefers_launcher());
        // Still has an executable of its own, so the user can switch back to it.
        assert!(!g.requires_launcher());
    }

    #[test]
    fn launch_url_format() {
        assert_eq!(
            test_game().launch_url(),
            "com.epicgames.launcher://apps/ns123%3Aitem456%3Atestgame123?action=launch&silent=true"
        );
    }

    #[test]
    fn dedupe_key_includes_all_parts() {
        assert_eq!(
            test_game().dedupe_key(),
            "/games/test-Binaries/Win64/game.exe-false"
        );
    }

    #[test]
    fn executable_path_joins_location_and_exe() {
        assert_eq!(
            test_game().executable_path(),
            PathBuf::from("/games/test/Binaries/Win64/game.exe")
        );
    }
}
