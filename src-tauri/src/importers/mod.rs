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
pub mod playnite;
#[cfg(windows)]
pub mod gamepass;

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
#[cfg(unix)]
pub mod proton;

use crate::{
    models::{ImportCandidate, ImportSource, SteamUser},
    steam::{artwork, non_steam_app_id},
};
use std::path::{Path, PathBuf};

pub fn quote_path(path: &Path) -> String {
    format!("\"{}\"", path.display())
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
    let start_dir = launcher_path.parent().map(PathBuf::from).unwrap_or_default();
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
    candidate
}
