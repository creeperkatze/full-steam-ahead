use crate::{
    error::AppResult,
    importers::{self, quote_path},
    models::{
        ArtworkKind, ArtworkSource, ImportCandidate, ImportSource, ScanRequest, ShortcutEntry,
        SteamUser,
    },
    steam::{artwork, non_steam_app_id},
};
use std::path::Path;

pub fn scan_sources(user: &SteamUser, request: &ScanRequest) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();
    let enabled_sources = enabled_sources(request);

    if cfg!(windows) && enabled_sources.contains(&ImportSource::Playnite) {
        candidates.extend(importers::playnite::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::Epic) {
        candidates.extend(importers::epic::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::Amazon) {
        candidates.extend(importers::amazon::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::Gog) {
        candidates.extend(importers::gog::scan(user)?);
    }

    if enabled_sources.contains(&ImportSource::Itch) {
        candidates.extend(importers::itch::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::Origin) {
        candidates.extend(importers::origin::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::UbisoftConnect) {
        candidates.extend(importers::ubisoft::scan(user)?);
    }

    if cfg!(windows) && enabled_sources.contains(&ImportSource::GamePass) {
        candidates.extend(importers::game_pass::scan(user)?);
    }

    Ok(candidates)
}

fn enabled_sources(request: &ScanRequest) -> Vec<ImportSource> {
    if !request.include_sources.is_empty() {
        return request.include_sources.clone();
    }

    vec![
        ImportSource::Playnite,
        ImportSource::Epic,
        ImportSource::Amazon,
        ImportSource::Gog,
        ImportSource::Itch,
        ImportSource::Origin,
        ImportSource::UbisoftConnect,
        ImportSource::GamePass,
    ]
}

pub fn shortcut_from_candidate(candidate: &ImportCandidate, grid_path: &Path) -> ShortcutEntry {
    ShortcutEntry {
        app_id: non_steam_app_id(&quote_path(&candidate.executable_path), &candidate.name),
        app_name: candidate.name.clone(),
        exe: quote_path(&candidate.executable_path),
        start_dir: quote_path(&candidate.start_dir),
        icon: shortcut_icon(candidate, grid_path),
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

fn shortcut_icon(candidate: &ImportCandidate, grid_path: &Path) -> String {
    let fallback = candidate.executable_path.display().to_string();
    let Some(asset) = artwork::selected_artwork_assets(candidate)
        .into_iter()
        .find(|asset| asset.kind == ArtworkKind::Icon)
    else {
        return fallback;
    };

    let icon_path = match asset.source {
        ArtworkSource::ExistingCustom => Path::new(&asset.path_or_url).to_path_buf(),
        ArtworkSource::OfficialSteam | ArtworkSource::SteamGridDb | ArtworkSource::LocalFile => {
            let app_id = non_steam_app_id(&quote_path(&candidate.executable_path), &candidate.name);
            artwork::target_path(grid_path, app_id, &ArtworkKind::Icon, &asset.path_or_url)
        }
    };

    if icon_path.exists() {
        icon_path.display().to_string()
    } else {
        fallback
    }
}
