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
