use crate::{
    error::{io_context, AppError, AppResult},
    importers::quote_path,
    models::{ArtworkKind, ShortcutEntry},
};
use std::{fs, path::Path};

const APP_NAME: &str = "Full Steam Ahead";

const HEADER: &[u8] = include_bytes!("../../assets/artwork/header.png");
const CAPSULE: &[u8] = include_bytes!("../../assets/artwork/capsule.png");
const HERO: &[u8] = include_bytes!("../../assets/artwork/hero.png");
const LOGO: &[u8] = include_bytes!("../../assets/artwork/logo.png");
const ICON: &[u8] = include_bytes!("../../assets/artwork/icon.png");

/// Builds the shortcut entry for Full Steam Ahead itself
pub fn build(grid_path: &Path) -> AppResult<ShortcutEntry> {
    let target = launch_target()?;
    let app_id = super::non_steam_app_id(&target.exe, APP_NAME);

    write_artwork(grid_path, app_id)?;

    Ok(ShortcutEntry {
        app_id,
        app_name: APP_NAME.to_string(),
        exe: target.exe,
        start_dir: target.start_dir,
        launch_options: target.launch_options,
        icon: super::artwork::target_path(grid_path, app_id, &ArtworkKind::Icon, "icon.png")
            .display()
            .to_string(),
        ..ShortcutEntry::default()
    })
}

struct LaunchTarget {
    exe: String,
    start_dir: String,
    launch_options: String,
}

fn launch_target() -> AppResult<LaunchTarget> {
    #[cfg(unix)]
    if let Some(app_id) = flatpak_app_id() {
        let flatpak = crate::importers::host_binary_path("flatpak");
        return Ok(host_target(flatpak, format!("run {app_id}")));
    }

    let exe = std::env::current_exe().map_err(|source| {
        AppError::Message(format!(
            "Failed to determine the current executable: {source}"
        ))
    })?;

    #[cfg(unix)]
    return Ok(host_target(exe, String::new()));

    #[cfg(not(unix))]
    {
        let start_dir = exe.parent().unwrap_or(Path::new("."));
        Ok(LaunchTarget {
            start_dir: quote_path(start_dir),
            exe: quote_path(&exe),
            launch_options: String::new(),
        })
    }
}

/// Routes through `flatpak-spawn --host` when Steam itself is sandboxed and cannot see us.
#[cfg(unix)]
fn host_target(exe: std::path::PathBuf, options: String) -> LaunchTarget {
    let (exe, launch_options) = crate::importers::host_launch(exe, options);
    let start_dir = exe.parent().unwrap_or(Path::new("/")).to_path_buf();

    LaunchTarget {
        exe: quote_path(&exe),
        start_dir: quote_path(&start_dir),
        launch_options,
    }
}

#[cfg(unix)]
fn flatpak_app_id() -> Option<String> {
    let info_path = Path::new("/.flatpak-info");
    if !info_path.exists() {
        return None;
    }

    if let Some(id) = std::env::var("FLATPAK_ID")
        .ok()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
    {
        return Some(id);
    }

    let info = fs::read_to_string(info_path)
        .inspect_err(|error| tracing::warn!(%error, "Could not read /.flatpak-info"))
        .ok()?;
    parse_flatpak_app_id(&info)
}

#[cfg(unix)]
fn parse_flatpak_app_id(info: &str) -> Option<String> {
    let mut in_application = false;

    for line in info.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_application = line == "[Application]";
            continue;
        }
        if !in_application {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "name" {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

fn write_artwork(grid_path: &Path, app_id: u32) -> AppResult<()> {
    fs::create_dir_all(grid_path).map_err(io_context(grid_path))?;

    for (kind, filename, bytes) in [
        (ArtworkKind::Header, "header.png", HEADER),
        (ArtworkKind::Capsule, "capsule.png", CAPSULE),
        (ArtworkKind::Hero, "hero.png", HERO),
        (ArtworkKind::Logo, "logo.png", LOGO),
        (ArtworkKind::Icon, "icon.png", ICON),
    ] {
        let target = super::artwork::target_path(grid_path, app_id, &kind, filename);
        fs::write(&target, bytes).map_err(io_context(&target))?;
    }

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn reads_app_id_from_application_section() {
        let info = "[Application]\nname=dev.creeperkatze.FullSteamAhead\nruntime=runtime/org.gnome.Platform/x86_64/47\n";
        assert_eq!(
            parse_flatpak_app_id(info).as_deref(),
            Some("dev.creeperkatze.FullSteamAhead")
        );
    }

    #[test]
    fn ignores_name_keys_outside_the_application_section() {
        let info = "[Instance]\nname=something-else\n\n[Application]\nname=dev.creeperkatze.FullSteamAhead\n";
        assert_eq!(
            parse_flatpak_app_id(info).as_deref(),
            Some("dev.creeperkatze.FullSteamAhead")
        );
    }

    #[test]
    fn returns_none_without_an_application_name() {
        let info = "[Instance]\nname=something-else\n\n[Application]\nruntime=runtime/org.gnome.Platform/x86_64/47\n";
        assert_eq!(parse_flatpak_app_id(info), None);
    }

    #[test]
    fn native_launch_target_points_at_the_executable() {
        if flatpak_app_id().is_some() {
            return; // Sandboxed test run takes the flatpak path instead.
        }

        let target = launch_target().unwrap();
        assert!(target.launch_options.is_empty());
        assert!(target.exe.starts_with('"') && target.exe.ends_with('"'));
    }
}
