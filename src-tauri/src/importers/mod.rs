pub mod epic;
pub mod gog;
pub mod itch;
pub mod manual;
pub mod origin;
pub mod ubisoft;

// Windows-only launchers
#[cfg(windows)]
pub mod amazon;
#[cfg(windows)]
pub mod gamepass;
#[cfg(windows)]
pub mod playnite;

// Unix-only launchers
#[cfg(unix)]
pub mod bottles;
#[cfg(unix)]
pub mod flatpak;
#[cfg(unix)]
pub mod heroic;
#[cfg(unix)]
pub mod legendary;
#[cfg(unix)]
pub mod lutris;
#[cfg(unix)]
pub mod minigalaxy;

use crate::{
    models::{ImportCandidate, ImportSource, SteamUser},
    steam::{artwork, non_steam_app_id},
};
use std::path::{Path, PathBuf};

pub fn quote_path(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

/// Returns the `steamapps/compatdata` directory under the detected Steam install.
#[cfg(unix)]
pub fn compat_data_dir() -> Option<PathBuf> {
    let install_path = crate::steam::detect::find_install_path()?;
    let compat_dir = install_path.join("steamapps").join("compatdata");
    compat_dir.exists().then_some(compat_dir)
}

/// Returns all Proton compat-data prefix paths found under the detected Steam install.
#[cfg(unix)]
pub fn find_proton_prefixes() -> Vec<PathBuf> {
    let Some(compat_dir) = compat_data_dir() else {
        return Vec::new();
    };
    std::fs::read_dir(&compat_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            path.join("pfx").exists().then_some(path)
        })
        .collect()
}

/// Translate a Windows-style path (e.g. `C:\Foo\Bar`) to a host path.
#[cfg(unix)]
pub fn translate_windows_path(compat_folder: &Path, windows_path: &str) -> Option<PathBuf> {
    let drive = windows_path.get(0..2).map(|d| d.to_lowercase())?;
    let rest = windows_path.get(3..)?.replace('\\', "/");
    Some(
        compat_folder
            .join("pfx")
            .join("dosdevices")
            .join(drive)
            .join(rest),
    )
}

pub fn candidate_from_parts(
    user: &SteamUser,
    source: ImportSource,
    source_slug: &str,
    name: String,
    executable_path: PathBuf,
    start_dir: PathBuf,
    launch_options: Option<String>,
    tags: Vec<String>,
) -> ImportCandidate {
    let app_id = non_steam_app_id(&quote_path(&executable_path), &name);
    let (matched_steam_app_id, artwork) =
        artwork::steam_preferred_plan(&user.grid_path, app_id, &name);

    ImportCandidate {
        id: format!("{source_slug}-{app_id}"),
        source,
        name,
        executable_path,
        start_dir,
        launch_options,
        existing_app_id: None,
        matched_steam_app_id,
        tags,
        artwork,
        url_scheme: None,
        launcher_path: None,
        use_launcher_url: false,
        needs_proton: false,
    }
}

pub fn launcher_candidate(
    user: &SteamUser,
    source: ImportSource,
    source_slug: &str,
    name: String,
    launcher_path: PathBuf,
    launch_url: String,
    tags: Vec<String>,
) -> ImportCandidate {
    let start_dir = launcher_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default();
    let mut candidate = candidate_from_parts(
        user,
        source,
        source_slug,
        name,
        launcher_path,
        start_dir,
        Some(launch_url.clone()),
        tags,
    );
    candidate.url_scheme = Some(launch_url);
    candidate.use_launcher_url = true;
    candidate
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn translates_c_drive_path() {
        let compat = Path::new("/home/user/.steam/compatdata/123");
        let result = translate_windows_path(compat, r"C:\Games\game.exe");
        assert_eq!(
            result,
            Some(PathBuf::from(
                "/home/user/.steam/compatdata/123/pfx/dosdevices/c:/Games/game.exe"
            ))
        );
    }

    #[test]
    fn lowercases_drive_letter() {
        let result = translate_windows_path(Path::new("/prefix"), r"D:\Games\game.exe");
        assert_eq!(
            result,
            Some(PathBuf::from("/prefix/pfx/dosdevices/d:/Games/game.exe"))
        );
    }

    #[test]
    fn empty_path_returns_none() {
        assert_eq!(translate_windows_path(Path::new("/prefix"), ""), None);
    }

    #[test]
    fn too_short_path_returns_none() {
        assert_eq!(translate_windows_path(Path::new("/prefix"), "C:"), None);
    }

    #[test]
    fn path_with_spaces_in_components() {
        let result = translate_windows_path(
            Path::new("/prefix"),
            r"C:\Program Files (x86)\Epic Games\launcher.exe",
        );
        assert_eq!(
            result,
            Some(PathBuf::from(
                "/prefix/pfx/dosdevices/c:/Program Files (x86)/Epic Games/launcher.exe"
            ))
        );
    }
}
