use std::path::{Path, PathBuf};

/// Returns the `steamapps/compatdata` directory under the detected Steam install.
pub fn compat_data_dir() -> Option<PathBuf> {
    let install_path = crate::steam::detect::find_install_path()?;
    let compat_dir = install_path.join("steamapps").join("compatdata");
    compat_dir.exists().then_some(compat_dir)
}

/// Returns all Proton compat-data prefix paths found under the detected Steam install.
pub fn find_proton_prefixes() -> Vec<PathBuf> {
    let Some(compat_dir) = compat_data_dir() else {
        return Vec::new();
    };
    std::fs::read_dir(&compat_dir)
        .inspect_err(|error| {
            tracing::warn!(path = %compat_dir.display(), %error, "Could not list Proton prefixes");
        })
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            path.join("pfx").exists().then_some(path)
        })
        .collect()
}

/// Registries of the Wine prefix containing `custom_path`, or of every Steam Proton prefix.
/// Only prefixes whose `system.reg` mentions `marker` are parsed.
pub fn wine_registries(
    custom_path: Option<&Path>,
    marker: &str,
) -> Vec<crate::util::registry::WineRegistry> {
    let prefixes: Vec<PathBuf> = match custom_path {
        Some(path) => path
            .ancestors()
            .find(|dir| dir.join("system.reg").exists())
            .map(Path::to_path_buf)
            .into_iter()
            .collect(),
        None => find_proton_prefixes()
            .into_iter()
            .map(|compat| compat.join("pfx"))
            .collect(),
    };
    let marker = marker.to_lowercase();
    prefixes
        .iter()
        .filter_map(|prefix| {
            let text = super::read_launcher_file(&prefix.join("system.reg"))?;
            text.to_lowercase()
                .contains(&marker)
                .then(|| crate::util::registry::WineRegistry::parse(prefix, &text))
        })
        .collect()
}

/// Launch options that open a launcher URL inside an existing Proton prefix.
pub fn proton_launch_options(compat_folder: &Path, url: &str) -> String {
    format!(
        "STEAM_COMPAT_DATA_PATH=\"{}\" %command% -{}",
        compat_folder.display(),
        super::shell_quote(url)
    )
}

/// Translate a Windows-style path (e.g. `C:\Foo\Bar`) to a host path.
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

#[cfg(test)]
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
