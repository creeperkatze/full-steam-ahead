use std::path::{Path, PathBuf};

/// Routes through `flatpak-spawn --host` when sandboxed, so host binaries stay reachable.
pub fn host_command(exe: &str) -> std::process::Command {
    host_command_impl(exe, Path::new("/.flatpak-info").exists())
}

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

static HOST_BINARIES: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, PathBuf>>,
> = std::sync::OnceLock::new();

fn resolve_host_binary(name: &str) -> Option<PathBuf> {
    let output = host_command("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .output()
        .inspect_err(|error| tracing::warn!(name, %error, "Could not look up command"))
        .ok()?;
    parse_command_v(&String::from_utf8_lossy(&output.stdout))
}

/// Path to `flatpak-spawn` inside a Flatpak sandbox, provided by every runtime.
const FLATPAK_SPAWN: &str = "/usr/bin/flatpak-spawn";

/// Rewrites a launch command so a Flatpak Steam can reach a binary on the host.
pub fn host_launch(exe: PathBuf, options: String) -> (PathBuf, String) {
    wrap_for_sandboxed_steam(exe, options, crate::steam::detect::is_sandboxed_steam())
}

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
fn parse_command_v(stdout: &str) -> Option<PathBuf> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('/'))
        .map(PathBuf::from)
}

#[cfg(test)]
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
}
