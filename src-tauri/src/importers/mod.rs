pub mod ea;
pub mod epic;
pub mod gog;
pub mod itch;
pub mod manual;
pub mod scan;
pub mod ubisoft;

// Windows-only launchers
#[cfg(windows)]
pub mod amazon;
#[cfg(windows)]
pub mod playnite;
#[cfg(windows)]
pub mod xbox;

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
pub mod sandbox;
#[cfg(unix)]
pub mod wine;

use crate::{
    models::{ImportCandidate, ImportSource, SteamUser},
    steam::{artwork, non_steam_app_id},
};
use std::path::{Path, PathBuf};

pub fn quote_path(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

/// Reads a launcher's file. A missing file is expected and only logged at debug.
pub fn read_launcher_file(path: &Path) -> Option<String> {
    read_launcher_file_bytes(path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Like [`read_launcher_file`], for files that aren't necessarily UTF-8.
pub fn read_launcher_file_bytes(path: &Path) -> Option<Vec<u8>> {
    match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(path = %path.display(), "File not found");
            None
        }
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "Could not read file");
            None
        }
    }
}

/// Parses JSON from a launcher and warns when it doesn't match.
pub fn parse_launcher_json<T: serde::de::DeserializeOwned>(origin: &str, raw: &str) -> Option<T> {
    serde_json::from_str(raw)
        .inspect_err(|error| {
            tracing::warn!(origin, %error, snippet = snippet(raw), "Unexpected JSON");
        })
        .ok()
}

/// Reads and parses a launcher's JSON file.
pub fn read_launcher_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let raw = read_launcher_file(path)?;
    parse_launcher_json(&path.display().to_string(), &raw)
}

/// Runs a launcher's CLI and returns its stdout.
#[cfg_attr(windows, allow(dead_code))]
pub fn command_stdout(command: &mut std::process::Command) -> Option<String> {
    command_output(command).map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Runs a launcher's CLI. A missing program is expected and only logged at debug.
pub fn command_output(command: &mut std::process::Command) -> Option<std::process::Output> {
    let description = format!("{command:?}");
    #[cfg(windows)]
    let result = crate::process::command_output_no_window(command);
    #[cfg(not(windows))]
    let result = command.output();
    match result {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(command = description, "Command not found");
            None
        }
        Err(error) => {
            tracing::warn!(command = description, %error, "Command could not be run");
            None
        }
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(
                command = description,
                status = %output.status,
                stderr = snippet(&stderr),
                "Command failed"
            );
            None
        }
        Ok(output) => Some(output),
    }
}

/// Keeps logged launcher output readable.
fn snippet(text: &str) -> &str {
    let text = text.trim();
    match text.char_indices().nth(500) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}

/// Shell-quotes a value for safe use in a Steam shortcut's LaunchOptions.
/// Steam shell-expands LaunchOptions around `%command%` on Linux.
pub fn shell_quote(value: &str) -> String {
    shlex::try_quote(value)
        .map(|quoted| quoted.into_owned())
        .unwrap_or_else(|_| "''".to_string())
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
        original_name: name.clone(),
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

/// Sandbox-wraps a launcher and its URL for candidates that can also start directly.
pub fn launcher_url_pair(launcher_path: PathBuf, launch_url: String) -> (PathBuf, String) {
    #[cfg(unix)]
    return sandbox::host_launch(launcher_path, launch_url);
    #[cfg(not(unix))]
    return (launcher_path, launch_url);
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
    // Launchers live on the host, which a Flatpak Steam can only reach via flatpak-spawn.
    #[cfg(unix)]
    let (launcher_path, launch_url) = sandbox::host_launch(launcher_path, launch_url);

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
