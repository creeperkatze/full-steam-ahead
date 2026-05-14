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
        use_url_launch: false,
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
