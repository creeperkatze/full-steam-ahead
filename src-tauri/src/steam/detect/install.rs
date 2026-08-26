use std::path::{Path, PathBuf};

pub(super) fn find_steam_install_path(override_path: Option<&Path>) -> Option<PathBuf> {
    // An explicit override is respected as-is: if it isn't a valid Steam install,
    // detection fails rather than silently falling back to auto-detection.
    if let Some(path) = override_path {
        return is_steam_install(path).then(|| path.to_path_buf());
    }

    platform_steam_install_path().or_else(common_steam_install_path)
}

pub(super) fn is_steam_install(path: &Path) -> bool {
    path.join("steamapps").is_dir() || path.join("config").is_dir()
}

#[cfg(windows)]
fn platform_steam_install_path() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Valve\\Steam").ok()?;
    let path: String = key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(path.replace('/', "\\")))
}

#[cfg(target_os = "macos")]
fn platform_steam_install_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("Steam")
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_steam_install_path() -> Option<PathBuf> {
    dirs::home_dir().and_then(|home| {
        let flatpak_home = home
            .join(".var")
            .join("app")
            .join("com.valvesoftware.Steam");

        [
            home.join(".steam").join("steam"),
            home.join(".steam").join("root"),
            home.join(".local").join("share").join("Steam"),
            home.join(".steam").join("debian-installation"),
            home.join("snap")
                .join("steam")
                .join("common")
                .join(".local")
                .join("share")
                .join("Steam"),
            flatpak_home.join("data").join("Steam"),
            flatpak_home.join(".local").join("share").join("Steam"),
            flatpak_home.join(".steam").join("steam"),
        ]
        .into_iter()
        .find(|path| is_steam_install(path))
    })
}

#[cfg(not(any(windows, unix)))]
fn platform_steam_install_path() -> Option<PathBuf> {
    None
}

fn common_steam_install_path() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|data| data.join("Steam"))
        .filter(|path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("fsa_install_test_{}_{}", std::process::id(), n));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn override_used_when_valid() {
        let dir = tmp_dir();
        fs::create_dir_all(dir.join("steamapps")).unwrap();

        assert_eq!(find_steam_install_path(Some(&dir)), Some(dir.clone()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_override_fails_instead_of_falling_back() {
        let dir = tmp_dir();
        // No steamapps/config subdir, so this isn't a valid Steam install.

        assert_eq!(find_steam_install_path(Some(&dir)), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
