use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use nom::{
    bytes::{
        complete::{tag, take_until, take_while},
        streaming::take,
    },
    multi::many0,
    AsChar, IResult, Parser,
};
use std::path::{Path, PathBuf};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let (launcher_path, db_path) = find_paths(custom_path)?;
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let bytes = std::fs::read(&db_path).map_err(|e| {
        if e.raw_os_error() == Some(32) {
            AppError::Message(
                "Playnite appears to be running and has locked its database. Close Playnite and try again.".to_string(),
            )
        } else {
            AppError::Io { path: db_path.clone(), source: e }
        }
    })?;

    let (_, games) = parse_db(&bytes).unwrap_or_default();

    let candidates = games
        .into_iter()
        .filter(|g| g.installed)
        .map(|game| {
            launcher_candidate(
                user,
                ImportSource::Playnite,
                "playnite",
                game.name,
                launcher_path.clone(),
                format!("--hidesplashscreen --start {}", game.id),
                vec!["Playnite".to_string()],
            )
        })
        .collect();

    Ok(candidates)
}

fn find_paths(custom_path: Option<&Path>) -> AppResult<(PathBuf, PathBuf)> {
    let install_dir = if let Some(custom) = custom_path {
        custom.to_path_buf()
    } else {
        let local = std::env::var("LOCALAPPDATA")
            .map_err(|_| AppError::Message("LOCALAPPDATA not set".to_string()))?;
        Path::new(&local).join("Playnite")
    };
    let launcher = install_dir.join("Playnite.DesktopApp.exe");
    if !launcher.exists() {
        return Err(AppError::Message("Playnite is not installed".to_string()));
    }
    let appdata =
        std::env::var("APPDATA").map_err(|_| AppError::Message("APPDATA not set".to_string()))?;

    // Playnite treats installs without an uninstaller as portable and keeps data beside the exe
    let config_root = if install_dir.join("unins000.exe").exists() {
        Path::new(&appdata).join("Playnite")
    } else {
        install_dir.clone()
    };
    let db_dir = std::fs::read_to_string(config_root.join("config.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<PlayniteConfig>(&raw).ok())
        .and_then(|c| c.database_path)
        .filter(|p| !p.trim().is_empty())
        .map(|p| expand_db_path(&p, &install_dir, &appdata))
        .unwrap_or_else(|| config_root.join("library"));
    Ok((launcher, db_dir.join("games.db")))
}

#[derive(serde::Deserialize)]
struct PlayniteConfig {
    #[serde(rename = "DatabasePath")]
    database_path: Option<String>,
}

/// Mirrors Playnite's `GameDatabase.GetFullDbPath`.
fn expand_db_path(path: &str, install_dir: &Path, appdata: &str) -> PathBuf {
    const PLAYNITE_DIR: &str = "{playnitedir}";
    const APPDATA: &str = "%appdata%";
    let lower = path.to_ascii_lowercase();
    let expanded = if let Some(i) = lower.find(PLAYNITE_DIR) {
        format!(
            "{}{}{}",
            &path[..i],
            install_dir.display(),
            &path[i + PLAYNITE_DIR.len()..]
        )
    } else if let Some(i) = lower.find(APPDATA) {
        format!("{}{}{}", &path[..i], appdata, &path[i + APPDATA.len()..])
    } else {
        path.to_string()
    };
    let expanded = PathBuf::from(expanded);
    if expanded.is_absolute() {
        expanded
    } else {
        install_dir.join(expanded)
    }
}

struct GameEntry {
    name: String,
    id: String,
    installed: bool,
}

fn parse_db(content: &[u8]) -> IResult<&[u8], Vec<GameEntry>> {
    many0(parse_game).parse(content)
}

fn parse_game(i: &[u8]) -> IResult<&[u8], GameEntry> {
    let (i, _) = take_until("_id")(i)?;
    let (i, _) = take_until("Image")(i)?;
    let (i, prefix_and_id) = take_until("\\")(i)?;
    let id_bytes = prefix_and_id
        .split(|b| *b == 0_u8)
        .next_back()
        .unwrap_or_default();
    let id = String::from_utf8_lossy(id_bytes).to_string();

    let (i, _) = take_until("IsInstalled")(i)?;
    let (i, _) = tag("IsInstalled")(i)?;
    let installed = matches!(i.get(1), Some(1u8));

    let (i, _) = take_until("InstallSizeGroup")(i)?;
    let (i, _) = take_until("Name")(i)?;
    let (i, _) = take(4usize)(i)?;
    let (i, _) = take_while(|b: u8| !b.is_alphanum())(i)?;
    let (i, name_bytes) = take_while(|b| b != 0)(i)?;
    let name = String::from_utf8_lossy(name_bytes).to_string();

    IResult::Ok((
        i,
        GameEntry {
            id,
            name,
            installed,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_bytes(id: &str, name: &str, installed: bool) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"SKIP_id_Image\x00\x00");
        v.extend_from_slice(id.as_bytes());
        v.push(b'\\');
        v.extend_from_slice(b"_IsInstalled\x00");
        v.push(if installed { 1u8 } else { 0u8 });
        v.extend_from_slice(b"_InstallSizeGroup_Name\x00\x00\x00\x00");
        v.extend_from_slice(name.as_bytes());
        v.push(0u8);
        v
    }

    #[test]
    fn parses_installed_game() {
        let data = game_bytes("abc-1234", "The Witcher 3", true);
        let (_, games) = parse_db(&data).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "abc-1234");
        assert_eq!(games[0].name, "The Witcher 3");
        assert!(games[0].installed);
    }

    #[test]
    fn parses_not_installed_game() {
        let data = game_bytes("xyz-5678", "Hades", false);
        let (_, games) = parse_db(&data).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Hades");
        assert!(!games[0].installed);
    }

    #[test]
    fn parses_multiple_games() {
        let mut data = game_bytes("id1", "Game One", true);
        data.extend(game_bytes("id2", "Game Two", false));
        let (_, games) = parse_db(&data).unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].name, "Game One");
        assert_eq!(games[1].name, "Game Two");
    }

    #[test]
    fn expands_playnite_dir_variable() {
        let path = expand_db_path(r"{PlayniteDir}\library", Path::new(r"D:\Playnite"), "");
        assert_eq!(path, PathBuf::from(r"D:\Playnite\library"));
    }

    #[test]
    fn expands_appdata_case_insensitively() {
        let path = expand_db_path(
            r"%APPDATA%\Playnite\library",
            Path::new(r"D:\Playnite"),
            r"C:\Users\me\AppData\Roaming",
        );
        assert_eq!(
            path,
            PathBuf::from(r"C:\Users\me\AppData\Roaming\Playnite\library")
        );
    }

    #[test]
    fn keeps_absolute_paths_and_resolves_relative_ones() {
        let install = Path::new(r"D:\Playnite");
        assert_eq!(
            expand_db_path(r"E:\PlayniteDb", install, ""),
            PathBuf::from(r"E:\PlayniteDb")
        );
        assert_eq!(
            expand_db_path("library", install, ""),
            PathBuf::from(r"D:\Playnite\library")
        );
    }

    #[test]
    fn empty_data_returns_empty() {
        let (_, games) = parse_db(b"no game data here").unwrap();
        assert!(games.is_empty());
    }
}
