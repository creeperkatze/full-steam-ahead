use std::path::PathBuf;

/// Returns all Proton compat-data prefix paths found under the default Steam location.
pub fn find_proton_prefixes() -> Vec<PathBuf> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    let compat_dir = PathBuf::from(&home)
        .join(".steam")
        .join("steam")
        .join("steamapps")
        .join("compatdata");
    if !compat_dir.exists() {
        return Vec::new();
    }
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

/// Translate a Windows-style path (e.g. `C:\Foo\Bar`) to a host path via the
/// `dosdevices` symlink tree inside the given Proton compat prefix.
pub fn translate_windows_path(compat_folder: &std::path::Path, windows_path: &str) -> Option<PathBuf> {
    let drive = windows_path.get(0..2).map(|d| d.to_lowercase())?;
    let rest = windows_path.get(3..)?.replace('\\', "/");
    Some(compat_folder.join("pfx").join("dosdevices").join(drive).join(rest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
}
