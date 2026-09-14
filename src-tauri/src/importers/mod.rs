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
mod icons;
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

/// Routes through `flatpak-spawn --host` when sandboxed, so host binaries stay reachable.
#[cfg(unix)]
pub fn host_command(exe: &str) -> std::process::Command {
    host_command_impl(exe, Path::new("/.flatpak-info").exists())
}

#[cfg(unix)]
fn host_command_impl(exe: &str, in_sandbox: bool) -> std::process::Command {
    if in_sandbox {
        let mut cmd = std::process::Command::new("flatpak-spawn");
        cmd.arg("--host").arg(exe);
        cmd
    } else {
        std::process::Command::new(exe)
    }
}

/// Resolves a bare command name to its absolute path on the host.
#[cfg(unix)]
pub fn host_binary_path(name: &str) -> PathBuf {
    if name.contains('/') {
        return PathBuf::from(name);
    }

    let cache = HOST_BINARIES.get_or_init(Default::default);
    if let Some(cached) = cache.lock().ok().and_then(|paths| paths.get(name).cloned()) {
        return cached;
    }

    let resolved =
        resolve_host_binary(name).unwrap_or_else(|| PathBuf::from("/usr/bin").join(name));
    if let Ok(mut paths) = cache.lock() {
        paths.insert(name.to_string(), resolved.clone());
    }
    resolved
}

#[cfg(unix)]
static HOST_BINARIES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, PathBuf>>,
> = std::sync::OnceLock::new();

#[cfg(unix)]
fn resolve_host_binary(name: &str) -> Option<PathBuf> {
    let output = host_command("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .output()
        .ok()?;
    parse_command_v(&String::from_utf8_lossy(&output.stdout))
}

/// Path to `flatpak-spawn` inside a Flatpak sandbox, provided by every runtime.
#[cfg(unix)]
const FLATPAK_SPAWN: &str = "/usr/bin/flatpak-spawn";

/// Rewrites a launch command so a Flatpak Steam can reach a binary on the host.
#[cfg(unix)]
pub fn host_launch(exe: PathBuf, options: String) -> (PathBuf, String) {
    wrap_for_sandboxed_steam(exe, options, crate::steam::detect::is_sandboxed_steam())
}

#[cfg(unix)]
fn wrap_for_sandboxed_steam(
    exe: PathBuf,
    options: String,
    steam_sandboxed: bool,
) -> (PathBuf, String) {
    if !steam_sandboxed {
        return (exe, options);
    }

    let exe = exe.display().to_string();
    let exe = if exe.contains(' ') {
        format!("\"{exe}\"")
    } else {
        exe
    };
    let options = if options.is_empty() {
        format!("--host {exe}")
    } else {
        format!("--host {exe} {options}")
    };

    (PathBuf::from(FLATPAK_SPAWN), options)
}

/// Takes the first absolute path
#[cfg(unix)]
fn parse_command_v(stdout: &str) -> Option<PathBuf> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('/'))
        .map(PathBuf::from)
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

/// Sandbox-wraps a launcher and its URL for candidates that can also start directly.
pub fn launcher_url_pair(launcher_path: PathBuf, launch_url: String) -> (PathBuf, String) {
    #[cfg(unix)]
    return host_launch(launcher_path, launch_url);
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
    let (launcher_path, launch_url) = host_launch(launcher_path, launch_url);

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
    fn host_command_runs_exe_directly_outside_sandbox() {
        let cmd = host_command_impl("flatpak", false);
        assert_eq!(cmd.get_program(), "flatpak");
        assert_eq!(cmd.get_args().count(), 0);
    }

    #[test]
    fn host_command_wraps_with_flatpak_spawn_inside_sandbox() {
        let cmd = host_command_impl("flatpak", true);
        assert_eq!(cmd.get_program(), "flatpak-spawn");
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, ["--host", "flatpak"]);
    }

    #[test]
    fn command_v_output_is_read_as_a_path() {
        assert_eq!(
            parse_command_v("/usr/bin/flatpak\n"),
            Some(PathBuf::from("/usr/bin/flatpak"))
        );
    }

    #[test]
    fn shell_builtins_and_empty_output_are_rejected() {
        // `command -v` prints the bare name for builtins and nothing for unknown commands
        assert_eq!(parse_command_v("flatpak\n"), None);
        assert_eq!(parse_command_v(""), None);
    }

    #[test]
    fn native_steam_launches_the_binary_directly() {
        let (exe, options) = wrap_for_sandboxed_steam(
            PathBuf::from("/usr/bin/flatpak"),
            "run com.foo.Bar".to_string(),
            false,
        );
        assert_eq!(exe, PathBuf::from("/usr/bin/flatpak"));
        assert_eq!(options, "run com.foo.Bar");
    }

    #[test]
    fn sandboxed_steam_breaks_out_to_the_host() {
        let (exe, options) = wrap_for_sandboxed_steam(
            PathBuf::from("/usr/bin/flatpak"),
            "run com.foo.Bar".to_string(),
            true,
        );
        assert_eq!(exe, PathBuf::from(FLATPAK_SPAWN));
        assert_eq!(options, "--host /usr/bin/flatpak run com.foo.Bar");
    }

    #[test]
    fn sandboxed_steam_handles_an_exe_without_options() {
        let (_, options) =
            wrap_for_sandboxed_steam(PathBuf::from("/usr/bin/lutris"), String::new(), true);
        assert_eq!(options, "--host /usr/bin/lutris");
    }

    #[test]
    fn sandboxed_steam_quotes_paths_containing_spaces() {
        let (_, options) = wrap_for_sandboxed_steam(
            PathBuf::from("/opt/my launcher/run"),
            "play".to_string(),
            true,
        );
        assert_eq!(options, "--host \"/opt/my launcher/run\" play");
    }

    #[test]
    fn explicit_paths_are_used_as_given() {
        assert_eq!(
            host_binary_path("/opt/custom/flatpak"),
            PathBuf::from("/opt/custom/flatpak")
        );
    }

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
