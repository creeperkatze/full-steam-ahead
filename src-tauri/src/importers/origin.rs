//! EA app games. Origin itself was shut down, but the EA app still handles `origin2://` links.

use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
    util::registry::Registry,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

const UNINSTALL_KEYS: [&str; 2] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
];
/// The `origin2://` handler takes the launch link. `eadm://` is the fallback.
const PROTOCOL_COMMAND_KEYS: [&str; 2] = [
    r"SOFTWARE\Classes\origin2\shell\open\command",
    r"SOFTWARE\Classes\eadm\shell\open\command",
];

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    #[cfg(windows)]
    {
        let registry = crate::util::registry::WindowsRegistry;
        let launcher = custom_path
            .map(|dir| dir.join("EADesktop.exe"))
            .filter(|exe| exe.exists())
            .or_else(|| launcher_path(&registry));
        let Some(launcher) = launcher else {
            tracing::debug!("EA app not found");
            return Ok(Vec::new());
        };
        tracing::debug!(launcher = %launcher.display(), "EA app found");
        Ok(scan_registry(user, &registry, &launcher, None))
    }

    #[cfg(unix)]
    {
        let mut candidates = Vec::new();
        for registry in super::wine_registries(custom_path, "EAInstaller") {
            let Some(launcher) = launcher_path(&registry) else {
                tracing::debug!(
                    prefix = %registry.prefix().display(),
                    "Prefix has EA installs but no EA app"
                );
                continue;
            };
            tracing::debug!(launcher = %launcher.display(), "EA app found");
            let compat = registry.proton_compat_folder();
            candidates.extend(scan_registry(user, &registry, &launcher, compat));
        }
        Ok(candidates)
    }

    #[cfg(not(any(windows, unix)))]
    Ok(Vec::new())
}

fn launcher_path(registry: &impl Registry) -> Option<PathBuf> {
    PROTOCOL_COMMAND_KEYS
        .iter()
        .filter_map(|key| registry.value(key, ""))
        .filter_map(|command| parse_quoted_executable(&command))
        .filter_map(|exe| registry.host_path(&exe))
        .find(|exe| exe.exists())
}

/// `compat_folder` is the Proton prefix the registry was read from, if any.
#[cfg_attr(windows, allow(unused_variables))]
fn scan_registry(
    user: &SteamUser,
    registry: &impl Registry,
    launcher: &Path,
    compat_folder: Option<&Path>,
) -> Vec<ImportCandidate> {
    // Keyed by content IDs, in case a game shows up in both registry views
    let mut games = BTreeMap::new();
    for root in UNINSTALL_KEYS {
        for subkey in registry.subkeys(root) {
            let key = format!(r"{root}\{subkey}");
            let Some(uninstall) = registry.value(&key, "UninstallString") else {
                continue;
            };
            // EA app installs register EAInstaller's Cleanup.exe as their uninstaller
            let uninstall = uninstall.to_lowercase();
            if !uninstall.contains("eainstaller") || !uninstall.contains("cleanup.exe") {
                continue;
            }
            let Some(install_dir) = registry
                .value(&key, "InstallLocation")
                .and_then(|dir| registry.host_path(&dir))
            else {
                tracing::debug!(key, "Skipping EA install without an install location");
                continue;
            };
            let Some(content_ids) = read_content_ids(&install_dir) else {
                continue;
            };
            let Some(title) = registry
                .value(&key, "DisplayName")
                .or_else(|| Some(install_dir.file_name()?.to_str()?.to_string()))
            else {
                tracing::debug!(key, "Skipping EA install without a name");
                continue;
            };
            games.insert(content_ids.join(","), title);
        }
    }

    games
        .into_iter()
        .map(|(offer_ids, title)| {
            let url = format!("origin2://game/launch?offerIds={offer_ids}&autoDownload=1");
            #[cfg(unix)]
            let url = match compat_folder {
                Some(compat) => super::proton_launch_options(compat, &url),
                None => url,
            };
            #[cfg_attr(not(unix), allow(unused_mut))]
            let mut candidate = launcher_candidate(
                user,
                ImportSource::Origin,
                "origin",
                title,
                launcher.to_path_buf(),
                url,
                vec!["EA app / Origin".to_string()],
            );
            // The EA app is a Windows program
            #[cfg(unix)]
            {
                candidate.needs_proton = true;
            }
            candidate
        })
        .collect()
}

/// Content IDs from the game's `__Installer/installerdata.xml` manifest.
fn read_content_ids(install_dir: &Path) -> Option<Vec<String>> {
    let path = install_dir.join("__Installer").join("installerdata.xml");
    let bytes = super::read_launcher_file_bytes(&path)?;
    parse_content_ids(&decode_text(&bytes))
        .inspect_err(|error| tracing::warn!(path = %path.display(), error, "Unusable EA manifest"))
        .ok()
}

fn parse_content_ids(xml: &str) -> Result<Vec<String>, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|error| error.to_string())?;
    let ids: Vec<String> = doc
        .descendants()
        .filter(|node| node.tag_name().name().eq_ignore_ascii_case("contentID"))
        .filter_map(|node| node.text())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    if ids.is_empty() {
        return Err("no content IDs".to_string());
    }
    Ok(ids)
}

/// Manifests are written as UTF-8 or UTF-16.
fn decode_text(bytes: &[u8]) -> String {
    // UTF-16 without a byte order mark starts with `<` and a zero byte.
    let guess = match bytes {
        [_, 0, ..] => encoding_rs::UTF_16LE,
        _ => encoding_rs::UTF_8,
    };
    // A byte order mark overrides the guess
    guess.decode(bytes).0.into_owned()
}

fn parse_quoted_executable(command: &str) -> Option<String> {
    if let Some(rest) = command.strip_prefix('"') {
        return Some(rest[..rest.find('"')?].to_string());
    }
    command.split_whitespace().next().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<DiPManifest version="4.0">
  <contentIDs>
    <contentID>1026023</contentID>
    <contentID>1035052</contentID>
  </contentIDs>
  <gameTitles><gameTitle locale="en_US">Battlefield 1</gameTitle></gameTitles>
</DiPManifest>"#;

    #[test]
    fn reads_all_content_ids() {
        assert_eq!(
            parse_content_ids(MANIFEST),
            Ok(vec!["1026023".to_string(), "1035052".to_string()])
        );
    }

    #[test]
    fn older_game_root_is_supported() {
        let xml = "<game><contentIDs><contentID> 71052 </contentID></contentIDs></game>";
        assert_eq!(parse_content_ids(xml), Ok(vec!["71052".to_string()]));
    }

    #[test]
    fn manifest_without_ids_is_rejected() {
        assert_eq!(
            parse_content_ids("<game><contentIDs/></game>"),
            Err("no content IDs".to_string())
        );
        assert!(parse_content_ids("not xml").is_err());
    }

    #[test]
    fn decodes_utf16_manifests() {
        let xml = r#"<?xml version="1.0" encoding="UTF-16"?><game><contentIDs><contentID>1</contentID></contentIDs></game>"#;
        let utf16: Vec<u8> = xml.encode_utf16().flat_map(u16::to_le_bytes).collect();

        let with_bom = [&[0xFF, 0xFE][..], &utf16].concat();
        let text = decode_text(&with_bom);
        assert_eq!(parse_content_ids(&text), Ok(vec!["1".to_string()]));
        assert_eq!(decode_text(&utf16), xml);

        let big_endian: Vec<u8> = [0xFE, 0xFF]
            .into_iter()
            .chain(xml.encode_utf16().flat_map(u16::to_be_bytes))
            .collect();
        assert_eq!(decode_text(&big_endian), xml);
    }

    #[test]
    fn decodes_utf8_manifests_with_and_without_bom() {
        let xml = "<game><contentIDs><contentID>1</contentID></contentIDs></game>";
        assert_eq!(decode_text(xml.as_bytes()), xml);
        let with_bom = [&[0xEF, 0xBB, 0xBF][..], xml.as_bytes()].concat();
        assert_eq!(decode_text(&with_bom), xml);
    }

    #[test]
    fn parse_quoted_exe_extracts_path() {
        assert_eq!(
            parse_quoted_executable(r#""C:\Program Files\EA\EALauncher.exe" "%1""#).as_deref(),
            Some(r"C:\Program Files\EA\EALauncher.exe")
        );
        assert_eq!(
            parse_quoted_executable(r"C:\EA\EA.exe %1").as_deref(),
            Some(r"C:\EA\EA.exe")
        );
        assert_eq!(parse_quoted_executable(""), None);
    }
}
