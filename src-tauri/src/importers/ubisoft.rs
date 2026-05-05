use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(launcher_key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher") else {
        return Ok(Vec::new());
    };
    let launcher_path = launcher_key
        .get_value::<String, _>("InstallDir")
        .ok()
        .and_then(|launcher_dir| launcher_from_dir(Path::new(&launcher_dir)))
        .or_else(default_launcher_path);
    let Some(launcher_path) = launcher_path else {
        return Ok(Vec::new());
    };

    let Ok(installs) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs")
    else {
        return Ok(Vec::new());
    };

    let mut candidates = Vec::new();
    for id in installs.enum_keys().flatten() {
        let Ok(install) = installs.open_subkey(&id) else {
            continue;
        };
        let Ok(install_dir): Result<String, _> = install.get_value("InstallDir") else {
            continue;
        };
        if !Path::new(&install_dir).exists() {
            continue;
        }

        let uninstall_path =
            format!("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Uplay Install {id}");
        let title = hklm
            .open_subkey(uninstall_path)
            .ok()
            .and_then(|key| key.get_value::<String, _>("DisplayName").ok())
            .unwrap_or_else(|| format!("Ubisoft game {id}"));
        candidates.push(launcher_candidate(
            user,
            ImportSource::UbisoftConnect,
            "ubisoft",
            title,
            launcher_path.clone(),
            format!("uplay://launch/{id}/0"),
            vec!["Ubisoft Connect".to_string()],
        ));
    }

    Ok(candidates)
}

fn launcher_from_dir(launcher_dir: &Path) -> Option<PathBuf> {
    ["UbisoftConnect.exe", "upc.exe"]
        .into_iter()
        .map(|name| launcher_dir.join(name))
        .find(|path| path.exists())
}

fn default_launcher_path() -> Option<PathBuf> {
    let program_files_x86 =
        std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
    launcher_from_dir(
        &Path::new(&program_files_x86)
            .join("Ubisoft")
            .join("Ubisoft Game Launcher"),
    )
}
