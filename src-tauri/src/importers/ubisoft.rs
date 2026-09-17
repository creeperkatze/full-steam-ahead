use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
    util::registry::Registry,
};
use std::path::{Path, PathBuf};

// Ubisoft Connect is a 32-bit app, so its keys live in the WOW64 view
const LAUNCHER_KEY: &str = r"SOFTWARE\WOW6432Node\Ubisoft\Launcher";
const INSTALLS_KEY: &str = r"SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs";
const UNINSTALL_KEY: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    #[cfg(windows)]
    {
        let registry = crate::util::registry::WindowsRegistry;
        let launcher = custom_path
            .and_then(launcher_from_dir)
            .or_else(|| launcher_path(&registry));
        Ok(launcher
            .map(|launcher| scan_registry(user, &registry, &launcher, None))
            .unwrap_or_default())
    }

    #[cfg(unix)]
    {
        let mut candidates = Vec::new();
        for registry in super::wine_registries(custom_path, "Ubisoft") {
            let Some(launcher) = launcher_path(&registry) else {
                continue;
            };
            let compat = registry.proton_compat_folder();
            candidates.extend(scan_registry(user, &registry, &launcher, compat));
        }
        Ok(candidates)
    }

    #[cfg(not(any(windows, unix)))]
    Ok(Vec::new())
}

fn launcher_path(registry: &impl Registry) -> Option<PathBuf> {
    let dir = registry.value(LAUNCHER_KEY, "InstallDir")?;
    launcher_from_dir(&registry.host_path(&dir)?)
}

fn launcher_from_dir(dir: &Path) -> Option<PathBuf> {
    ["UbisoftConnect.exe", "upc.exe"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.exists())
}

/// `compat_folder` is the Proton prefix the registry was read from, if any.
#[cfg_attr(windows, allow(unused_variables))]
fn scan_registry(
    user: &SteamUser,
    registry: &impl Registry,
    launcher: &Path,
    compat_folder: Option<&Path>,
) -> Vec<ImportCandidate> {
    registry
        .subkeys(INSTALLS_KEY)
        .into_iter()
        .filter_map(|id| {
            let install_dir = registry.value(&format!(r"{INSTALLS_KEY}\{id}"), "InstallDir")?;
            let install_dir = registry.host_path(&install_dir)?;
            if !install_dir.exists() {
                return None;
            }
            let title = registry
                .value(
                    &format!(r"{UNINSTALL_KEY}\Uplay Install {id}"),
                    "DisplayName",
                )
                .or_else(|| Some(install_dir.file_name()?.to_str()?.to_string()))?;
            let url = format!("uplay://launch/{id}/0");
            #[cfg(unix)]
            let url = match compat_folder {
                Some(compat) => super::proton_launch_options(compat, &url),
                None => url,
            };
            #[cfg_attr(not(unix), allow(unused_mut))]
            let mut candidate = launcher_candidate(
                user,
                ImportSource::UbisoftConnect,
                "ubisoft",
                title,
                launcher.to_path_buf(),
                url,
                vec!["Ubisoft Connect".to_string()],
            );
            // The launcher is a Windows program
            #[cfg(unix)]
            {
                candidate.needs_proton = true;
            }
            Some(candidate)
        })
        .collect()
}
