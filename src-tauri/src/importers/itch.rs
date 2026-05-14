use crate::{
    error::{AppError, AppResult},
    importers::candidate_from_parts,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use flate2::read::GzDecoder;
use nom::{
    bytes::complete::{tag, take_until},
    multi::many0,
    IResult,
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    io::Read,
    path::{Path, PathBuf},
};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let itch_dir = default_itch_location();
    let wal_path = itch_dir.join("db").join("butler.db-wal");
    if !wal_path.exists() {
        return Ok(Vec::new());
    }

    let bytes = std::fs::read(&wal_path).map_err(|source| AppError::Io {
        path: wal_path,
        source,
    })?;

    let (_, db_paths) = parse_butler_db(&bytes).unwrap_or_default();

    let unique: HashSet<DbPaths> = db_paths.into_iter().collect();

    let mut candidates = Vec::new();
    for db_path in &unique {
        if let Some(candidate) = db_path_to_candidate(user, db_path) {
            candidates.push(candidate);
        }
    }

    Ok(candidates)
}

fn db_path_to_candidate(user: &SteamUser, db_path: &DbPaths) -> Option<ImportCandidate> {
    let base = Path::new(&db_path.base_path);

    let executable_rel = db_path
        .paths
        .iter()
        .find(|p| is_executable(&base.join(p)))?;

    let executable_path = base.join(executable_rel);
    let start_dir = PathBuf::from(&db_path.base_path);

    // Title from gzip receipt first, plain receipt second, directory name last
    let receipt_gz = base.join(".itch").join("receipt.json.gz");
    let receipt_plain = base.join(".itch").join("receipt.json");
    let title = read_title_gz(&receipt_gz)
        .or_else(|| read_title_plain(&receipt_plain))
        .unwrap_or_else(|| {
            base.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("itch.io game")
                .to_string()
        });

    Some(candidate_from_parts(
        user,
        ImportSource::Itch,
        "itch",
        title,
        executable_path,
        start_dir,
        None,
        vec!["itch.io".to_string()],
    ))
}

fn read_title_gz(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut s = String::new();
    decoder.read_to_string(&mut s).ok()?;
    serde_json::from_str::<Receipt>(&s)
        .ok()
        .map(|r| r.game.title)
}

fn read_title_plain(path: &Path) -> Option<String> {
    let s = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<Receipt>(&s)
        .ok()
        .map(|r| r.game.title)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("exe" | "bat" | "cmd")
    )
}

fn default_itch_location() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .map(|d| PathBuf::from(d).join("itch"))
            .unwrap_or_else(|_| PathBuf::from("itch"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .map(|d| d.join("itch"))
            .unwrap_or_else(|| PathBuf::from("itch"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".config").join("itch"))
            .unwrap_or_else(|_| PathBuf::from(".config/itch"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DbPaths {
    base_path: String,
    paths: Vec<String>,
}

#[derive(Deserialize)]
struct Candidate {
    path: String,
}

fn parse_butler_db(content: &[u8]) -> IResult<&[u8], Vec<DbPaths>> {
    many0(parse_db_entry)(content)
}

fn parse_db_entry(i: &[u8]) -> IResult<&[u8], DbPaths> {
    let (i, _) = take_until("{\"basePath\":\"")(i)?;
    let (i, _) = tag("{\"basePath\":\"")(i)?;
    let (i, base_path_bytes) = take_until("\",\"totalSize\"")(i)?;
    let base_path = String::from_utf8_lossy(base_path_bytes).to_string();

    let (i, _) = take_until("\"candidates\":[")(i)?;
    let (i, _) = tag("\"candidates\":[")(i)?;
    let (i, cand_bytes) = take_until("]}")(i)?;
    let cand_json = format!("[{}]", String::from_utf8_lossy(cand_bytes));

    let paths = serde_json::from_str::<Vec<Candidate>>(&cand_json)
        .map(|v| v.into_iter().map(|c| c.path).collect())
        .unwrap_or_default();

    Ok((i, DbPaths { base_path, paths }))
}

#[derive(Deserialize)]
struct Receipt {
    game: ReceiptGame,
}

#[derive(Deserialize)]
struct ReceiptGame {
    title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_entry() {
        let data = b"junk{\"basePath\":\"/games/mygame\",\"totalSize\":100,\"candidates\":[{\"path\":\"mygame.exe\"}]}trailing";
        let (_, entries) = parse_butler_db(data).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].base_path, "/games/mygame");
        assert_eq!(entries[0].paths, vec!["mygame.exe"]);
    }

    #[test]
    fn parse_multiple_entries() {
        let data = b"{\"basePath\":\"/game1\",\"totalSize\":1,\"candidates\":[{\"path\":\"a.exe\"}]}{\"basePath\":\"/game2\",\"totalSize\":1,\"candidates\":[{\"path\":\"b.exe\"}]}";
        let (_, entries) = parse_butler_db(data).unwrap();
        assert_eq!(entries.len(), 2);
        let bases: Vec<&str> = entries.iter().map(|e| e.base_path.as_str()).collect();
        assert!(bases.contains(&"/game1"));
        assert!(bases.contains(&"/game2"));
    }

    #[test]
    fn parse_multiple_candidates_per_entry() {
        let data = b"{\"basePath\":\"/game\",\"totalSize\":1,\"candidates\":[{\"path\":\"a.exe\"},{\"path\":\"b.sh\"}]}";
        let (_, entries) = parse_butler_db(data).unwrap();
        assert_eq!(entries[0].paths, vec!["a.exe", "b.sh"]);
    }

    #[test]
    fn parse_no_matching_data_returns_empty() {
        let (_, entries) = parse_butler_db(b"no butler data here").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_entry_with_empty_candidates() {
        let data = b"{\"basePath\":\"/game\",\"totalSize\":0,\"candidates\":[]}";
        let (_, entries) = parse_butler_db(data).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].paths.is_empty());
    }

    #[cfg(not(unix))]
    #[test]
    fn executable_by_extension() {
        assert!(is_executable(Path::new("game.exe")));
        assert!(is_executable(Path::new("script.bat")));
        assert!(is_executable(Path::new("run.cmd")));
        assert!(!is_executable(Path::new("readme.txt")));
        assert!(!is_executable(Path::new("game")));
    }
}
