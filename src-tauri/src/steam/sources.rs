use crate::{
    error::AppResult,
    importers::{self, quote_path},
    models::{ImportCandidate, ScanRequest, ShortcutEntry, SteamUser},
    steam::non_steam_app_id,
};

pub fn scan_sources(user: &SteamUser, request: &ScanRequest) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();

    if request.include_playnite {
        candidates.extend(importers::playnite::scan(user)?);
    }

    if request.include_epic {
        candidates.extend(importers::epic::scan(user)?);
    }

    Ok(candidates)
}

pub fn shortcut_from_candidate(candidate: &ImportCandidate) -> ShortcutEntry {
    ShortcutEntry {
        app_id: non_steam_app_id(&quote_path(&candidate.executable_path), &candidate.name),
        app_name: candidate.name.clone(),
        exe: quote_path(&candidate.executable_path),
        start_dir: quote_path(&candidate.start_dir),
        icon: String::new(),
        shortcut_path: String::new(),
        launch_options: candidate.launch_options.clone().unwrap_or_default(),
        is_hidden: false,
        allow_desktop_config: true,
        allow_overlay: true,
        open_vr: false,
        devkit: false,
        devkit_game_id: String::new(),
        last_play_time: 0,
        tags: candidate.tags.clone(),
    }
}
