use crate::{
    error::{io_context, AppResult},
    importers::candidate_from_parts,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let mut all_candidates = Vec::new();

    for source in find_galaxy_configs() {
        let Ok(raw) = fs::read_to_string(&source.config_path) else {
            continue;
        };
        let Ok(config) = serde_json::from_str::<GogConfig>(&raw) else {
            continue;
        };

        let mut roots = config.installation_paths.unwrap_or_default();
        if roots.is_empty() {
            if let Some(path) = config.library_path {
                roots.push(path);
            }
        }

        // On Unix, translate Windows-style C:\ paths to the wine_c_drive equivalent
        #[cfg(unix)]
        let roots: Vec<String> = if let Some(ref wine_c) = source.wine_c_drive {
            roots
                .into_iter()
                .filter_map(|path| translate_installation_path(&path, wine_c))
                .collect()
        } else {
            roots
        };

        for root in roots {
            let root = PathBuf::from(root);
            if !root.exists() {
                continue;
            }
            for entry in fs::read_dir(&root).map_err(io_context(&root))?.flatten() {
                let folder = entry.path();
                if !folder.is_dir() {
                    continue;
                }
                let game_folder = if folder.join("game").exists() {
                    folder.join("game")
                } else {
                    folder
                };
                let mut found =
                    scan_gog_folder(user, &game_folder, source.compat_folder.as_deref())?;
                // Ensures Steam launches this shortcut through Proton, since it was found in a Wine/Proton prefix.
                if source.wine_c_drive.is_some() {
                    for candidate in &mut found {
                        candidate.needs_proton = true;
                    }
                }
                all_candidates.extend(found);
            }
        }
    }

    Ok(all_candidates)
}

#[cfg(unix)]
pub fn scan_folders(user: &SteamUser, folders: Vec<PathBuf>) -> Vec<ImportCandidate> {
    folders
        .into_iter()
        .flat_map(|f| scan_gog_folder(user, &f, None).unwrap_or_default())
        .collect()
}

fn scan_gog_folder(
    user: &SteamUser,
    game_folder: &Path,
    compat_folder: Option<&Path>,
) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(game_folder)
        .map_err(io_context(game_folder))?
        .flatten()
    {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !file_name.starts_with("goggame-")
            || path.extension().and_then(|e| e.to_str()) != Some("info")
        {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(io_context(&path))?;
        let Ok(game) = serde_json::from_str::<GogGame>(&raw) else {
            continue;
        };
        let Some(task) = game.play_tasks.unwrap_or_default().into_iter().find(|t| {
            t.is_primary.unwrap_or_default()
                && t.task_type == "FileTask"
                && matches!(t.category.as_deref(), Some("launcher" | "game"))
                && t.path.is_some()
        }) else {
            continue;
        };

        let exe_path = game_folder.join(task.path.unwrap_or_default());
        let work_dir = task
            .working_dir
            .map(|d| game_folder.join(d))
            .unwrap_or_else(|| game_folder.to_path_buf());

        // On Unix, normalise backslashes left over from Windows-style path components
        #[cfg(unix)]
        let exe_path = PathBuf::from(exe_path.to_string_lossy().replace('\\', "/"));
        #[cfg(unix)]
        let work_dir = PathBuf::from(work_dir.to_string_lossy().replace('\\', "/"));

        let args = task.arguments.filter(|a| !a.trim().is_empty());
        // Ensures Steam reuses the exact Proton prefix GOG Galaxy installed this game into.
        let args = match compat_folder {
            Some(compat) => Some(steam_compat_data_launch_options(compat, args.as_deref())),
            None => args,
        };
        candidates.push(candidate_from_parts(
            user,
            ImportSource::Gog,
            "gog",
            game.name,
            exe_path,
            work_dir,
            args,
            vec!["GOG".to_string()],
        ));
    }
    Ok(candidates)
}

fn steam_compat_data_launch_options(compat_folder: &Path, extra_args: Option<&str>) -> String {
    let mut options = format!(
        "STEAM_COMPAT_DATA_PATH=\"{}\" %command%",
        compat_folder.display()
    );
    if let Some(extra_args) = extra_args.filter(|a| !a.trim().is_empty()) {
        options.push(' ');
        options.push_str(extra_args);
    }
    options
}

struct GalaxyConfigSource {
    config_path: PathBuf,
    wine_c_drive: Option<PathBuf>,
    /// Set only when found inside a Steam-managed Proton prefix, not an external Wine prefix.
    compat_folder: Option<PathBuf>,
}

fn find_galaxy_configs() -> Vec<GalaxyConfigSource> {
    #[cfg(windows)]
    {
        let base = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        let config_path = PathBuf::from(base)
            .join("GOG.com")
            .join("Galaxy")
            .join("config.json");
        vec![GalaxyConfigSource {
            config_path,
            wine_c_drive: None,
            compat_folder: None,
        }]
    }

    #[cfg(unix)]
    {
        let mut result = Vec::new();
        let Ok(home) = std::env::var("HOME") else {
            return result;
        };

        // Default PlayOnLinux / Lutris GOG Galaxy location
        let default_drive_c = PathBuf::from(&home)
            .join("Games")
            .join("gog-galaxy")
            .join("drive_c");
        let default_config = default_drive_c
            .join("ProgramData")
            .join("GOG.com")
            .join("Galaxy")
            .join("config.json");
        if default_config.exists() {
            result.push(GalaxyConfigSource {
                config_path: default_config,
                wine_c_drive: Some(default_drive_c),
                compat_folder: None,
            });
        }

        // Proton compat data prefixes
        for prefix in super::find_proton_prefixes() {
            let drive_c = prefix.join("pfx").join("drive_c");
            let config_path = drive_c
                .join("ProgramData")
                .join("GOG.com")
                .join("Galaxy")
                .join("config.json");
            if config_path.exists() {
                result.push(GalaxyConfigSource {
                    config_path,
                    wine_c_drive: Some(drive_c),
                    compat_folder: Some(prefix),
                });
            }
        }

        result
    }
}

#[cfg(unix)]
fn translate_installation_path(path: &str, wine_c_drive: &Path) -> Option<String> {
    let stripped = path.strip_prefix("C:\\")?;
    let translated = wine_c_drive.join(stripped);
    Some(translated.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn compat_data_launch_options_without_args() {
        let compat = Path::new("/home/user/.steam/steam/steamapps/compatdata/123");
        assert_eq!(
            steam_compat_data_launch_options(compat, None),
            "STEAM_COMPAT_DATA_PATH=\"/home/user/.steam/steam/steamapps/compatdata/123\" %command%"
        );
    }

    #[test]
    fn compat_data_launch_options_appends_extra_args() {
        let compat = Path::new("/prefix/123");
        assert_eq!(
            steam_compat_data_launch_options(compat, Some("-windowed -skipintro")),
            "STEAM_COMPAT_DATA_PATH=\"/prefix/123\" %command% -windowed -skipintro"
        );
    }

    #[test]
    fn compat_data_launch_options_ignores_blank_extra_args() {
        let compat = Path::new("/prefix/123");
        assert_eq!(
            steam_compat_data_launch_options(compat, Some("   ")),
            "STEAM_COMPAT_DATA_PATH=\"/prefix/123\" %command%"
        );
    }

    #[cfg(unix)]
    #[test]
    fn translates_c_drive_path() {
        let wine_c = Path::new("/home/user/Games/gog-galaxy/drive_c");
        let result = translate_installation_path(r"C:\GOG Games\Witcher 3", wine_c);
        assert_eq!(
            result,
            Some("/home/user/Games/gog-galaxy/drive_c/GOG Games/Witcher 3".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_c_drive_returns_none() {
        let wine_c = Path::new("/home/user/drive_c");
        assert_eq!(
            translate_installation_path(r"D:\Games\Witcher 3", wine_c),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn deeply_nested_path() {
        let wine_c = Path::new("/prefix");
        let result =
            translate_installation_path(r"C:\Games\Publisher\Studio\Title\v2", wine_c).unwrap();
        assert!(result.ends_with("Games/Publisher/Studio/Title/v2"));
        assert!(!result.contains('\\'));
    }

    #[cfg(unix)]
    #[test]
    fn backslashes_converted_to_forward_slashes() {
        let wine_c = Path::new("/prefix");
        let result = translate_installation_path(r"C:\Games\Studio\MyGame", wine_c).unwrap();
        assert!(!result.contains('\\'));
        assert!(result.ends_with("Games/Studio/MyGame"));
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogConfig {
    installation_paths: Option<Vec<String>>,
    library_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogGame {
    name: String,
    play_tasks: Option<Vec<GogPlayTask>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GogPlayTask {
    category: Option<String>,
    is_primary: Option<bool>,
    path: Option<String>,
    #[serde(rename = "type")]
    task_type: String,
    working_dir: Option<String>,
    arguments: Option<String>,
}
