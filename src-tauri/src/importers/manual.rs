use crate::{
    importers::quote_path,
    models::{ImportCandidate, ManualImportRequest},
    steam::{artwork, non_steam_app_id},
};
use std::path::{Path, PathBuf};

pub fn candidate(request: ManualImportRequest) -> ImportCandidate {
    let executable_path = request.executable_path;
    let name = request.display_name.unwrap_or_else(|| {
        executable_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Untitled Game")
            .to_string()
    });
    let start_dir = executable_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default();
    let app_id = non_steam_app_id(&quote_path(&executable_path), &name);

    ImportCandidate {
        id: format!("manual-{app_id}"),
        source: request.source,
        name,
        executable_path,
        start_dir,
        launch_options: None,
        existing_app_id: None,
        matched_steam_app_id: None,
        tags: request.tags,
        artwork: artwork::preserve_existing_plan(Path::new(""), app_id),
        url_scheme: None,
        launcher_path: None,
        use_launcher_url: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ImportSource;

    fn request(exe: &str, display_name: Option<&str>) -> ManualImportRequest {
        ManualImportRequest {
            user_steam_id: "test".to_string(),
            executable_path: PathBuf::from(exe),
            display_name: display_name.map(String::from),
            source: ImportSource::Manual,
            tags: vec![],
        }
    }

    #[test]
    fn uses_display_name_when_provided() {
        let c = candidate(request("game.exe", Some("My Game")));
        assert_eq!(c.name, "My Game");
    }

    #[test]
    fn falls_back_to_file_stem() {
        let c = candidate(request("game.exe", None));
        assert_eq!(c.name, "game");
    }

    #[test]
    fn falls_back_to_untitled_when_no_stem() {
        let c = candidate(request("", None));
        assert_eq!(c.name, "Untitled Game");
    }

    #[test]
    fn start_dir_is_parent_of_executable() {
        let c = candidate(request("C:/Games/MyGame/game.exe", None));
        assert_eq!(c.start_dir, PathBuf::from("C:/Games/MyGame"));
    }

    #[test]
    fn id_prefixed_with_manual() {
        let c = candidate(request("game.exe", Some("Test")));
        assert!(c.id.starts_with("manual-"));
    }

    #[test]
    fn source_preserved_from_request() {
        let mut req = request("game.exe", None);
        req.source = ImportSource::Epic;
        let c = candidate(req);
        assert_eq!(c.source, ImportSource::Epic);
    }

    #[test]
    fn tags_preserved_from_request() {
        let mut req = request("game.exe", None);
        req.tags = vec!["Custom".to_string()];
        let c = candidate(req);
        assert_eq!(c.tags, vec!["Custom"]);
    }
}

pub fn candidate_with_grid_path(request: ManualImportRequest, grid_path: &Path) -> ImportCandidate {
    let mut candidate = candidate(request);
    let app_id = non_steam_app_id(&quote_path(&candidate.executable_path), &candidate.name);
    let (matched_steam_app_id, artwork) =
        artwork::steam_preferred_plan(grid_path, app_id, &candidate.name);
    candidate.matched_steam_app_id = matched_steam_app_id;
    candidate.artwork = artwork;
    candidate
}
