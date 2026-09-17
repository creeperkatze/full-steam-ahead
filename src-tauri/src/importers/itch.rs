use crate::{
    error::{AppError, AppResult},
    importers::candidate_from_parts,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use sqlite::{OpenFlags, State};
use std::path::{Path, PathBuf};

/// Every install ("cave") butler knows about, with its game title when the game is known.
const CAVES_QUERY: &str =
    "SELECT games.title, caves.verdict FROM caves LEFT JOIN games ON games.id = caves.game_id";

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let itch_dir = custom_path
        .map(PathBuf::from)
        .unwrap_or_else(default_itch_location);
    let db_path = itch_dir.join("db").join("butler.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let db_error = |error: sqlite::Error| {
        AppError::Message(format!(
            "Could not read itch.io database at {}: {error}",
            db_path.display()
        ))
    };
    // SQLite picks up writes still sitting in butler's WAL file on its own
    let connection =
        sqlite::Connection::open_with_flags(&db_path, OpenFlags::new().with_read_only())
            .map_err(db_error)?;
    let mut statement = connection.prepare(CAVES_QUERY).map_err(db_error)?;

    let mut candidates = Vec::new();
    while let Ok(State::Row) = statement.next() {
        let title = statement.read::<Option<String>, _>(0).ok().flatten();
        let Ok(Some(verdict)) = statement.read::<Option<String>, _>(1) else {
            continue;
        };
        let Ok(verdict) = serde_json::from_str::<Verdict>(&verdict) else {
            continue;
        };
        candidates.extend(verdict_to_candidate(user, title, verdict));
    }

    Ok(candidates)
}

/// A cave's `verdict` column, as serialized by itchio/dash.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Verdict {
    base_path: String,
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    path: String,
}

fn verdict_to_candidate(
    user: &SteamUser,
    title: Option<String>,
    verdict: Verdict,
) -> Option<ImportCandidate> {
    let base = PathBuf::from(&verdict.base_path);
    // Butler lists the best launch target first
    let executable_path = verdict
        .candidates
        .unwrap_or_default()
        .into_iter()
        .map(|c| base.join(c.path))
        .find(|p| is_executable(p))?;

    // Linked folders may have no game row
    let title = title.or_else(|| base.file_name()?.to_str().map(str::to_string))?;

    Some(candidate_from_parts(
        user,
        ImportSource::Itch,
        "itch",
        title,
        executable_path,
        base,
        None,
        vec!["itch.io".to_string()],
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verdict_with_extra_fields() {
        let json = r#"{"basePath":"/games/mygame","totalSize":100,"candidates":[
            {"path":"bin/game","depth":2,"flavor":"linux","size":5,"spell":["ELF executable"]},
            {"path":"game.exe","windowsInfo":{"gui":true}}
        ]}"#;
        let verdict: Verdict = serde_json::from_str(json).unwrap();
        assert_eq!(verdict.base_path, "/games/mygame");
        let paths: Vec<_> = verdict
            .candidates
            .unwrap()
            .into_iter()
            .map(|c| c.path)
            .collect();
        assert_eq!(paths, ["bin/game", "game.exe"]);
    }

    #[test]
    fn parses_verdict_with_null_candidates() {
        let verdict: Verdict =
            serde_json::from_str(r#"{"basePath":"/game","totalSize":0,"candidates":null}"#)
                .unwrap();
        assert!(verdict.candidates.is_none());
    }

    #[test]
    fn reads_caves_from_butler_schema() {
        let db = sqlite::open(":memory:").unwrap();
        db.execute(
            r#"
            CREATE TABLE games (id INTEGER PRIMARY KEY, title TEXT);
            CREATE TABLE caves (id TEXT PRIMARY KEY, game_id INTEGER, verdict TEXT);
            INSERT INTO games VALUES (1, 'Known Game');
            INSERT INTO caves VALUES ('a', 1, '{"basePath":"/a","candidates":[]}');
            INSERT INTO caves VALUES ('b', 0, '{"basePath":"/b","candidates":[]}');
            "#,
        )
        .unwrap();
        let mut statement = db.prepare(CAVES_QUERY).unwrap();
        let mut rows = Vec::new();
        while let Ok(State::Row) = statement.next() {
            rows.push(statement.read::<Option<String>, _>(0).unwrap());
        }
        assert_eq!(rows, [Some("Known Game".to_string()), None]);
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
