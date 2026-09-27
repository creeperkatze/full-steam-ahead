use crate::models::{ImportCandidate, ManualImportRequest, SteamUser};
use std::path::{Path, PathBuf};

pub fn candidate(user: &SteamUser, request: ManualImportRequest) -> ImportCandidate {
    let name = request
        .display_name
        .unwrap_or_else(|| name_from_path(&request.executable_path));
    let start_dir = request
        .executable_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default();
    super::candidate_from_parts(
        user,
        request.source,
        "manual",
        name,
        request.executable_path,
        start_dir,
        None,
        request.tags,
    )
}

fn name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled Game")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_falls_back_to_file_stem() {
        assert_eq!(name_from_path(Path::new("C:/Games/game.exe")), "game");
    }

    #[test]
    fn name_falls_back_to_untitled_when_no_stem() {
        assert_eq!(name_from_path(Path::new("")), "Untitled Game");
    }
}
