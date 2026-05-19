use std::path::PathBuf;

pub(super) fn find_steam_install_path() -> Option<PathBuf> {
    platform_steam_install_path().or_else(common_steam_install_path)
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
        [
            home.join(".steam").join("steam"),
            home.join(".local").join("share").join("Steam"),
            home.join(".var")
                .join("app")
                .join("com.valvesoftware.Steam")
                .join(".steam")
                .join("steam"),
        ]
        .into_iter()
        .find(|path| path.exists())
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
