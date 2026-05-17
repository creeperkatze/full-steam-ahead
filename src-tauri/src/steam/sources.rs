use crate::{
    error::AppResult,
    importers::{self, quote_path},
    models::{
        ArtworkKind, ArtworkSource, ImportCandidate, ImportSource, ScanProgressEvent, ScanRequest,
        ShortcutEntry, SteamUser,
    },
    steam::{artwork, non_steam_app_id},
};
use std::path::Path;
use tauri::Emitter;

pub fn scan_sources_with_progress(
    app: &tauri::AppHandle,
    user: &SteamUser,
    request: &ScanRequest,
) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();
    let enabled_sources = enabled_sources(request);

    for source in &enabled_sources {
        let _ = app.emit(
            "scan-progress",
            ScanProgressEvent {
                source: source.clone(),
                status: "scanning".to_string(),
                found: 0,
            },
        );

        let found = scan_single_source(source, user);
        let found_count = found.len();
        candidates.extend(found);

        if found_count == 0 {
            tracing::debug!(source = %source.display_name(), "No games found");
        } else {
            tracing::info!(source = %source.display_name(), found = found_count, "Games found");
        }

        let _ = app.emit(
            "scan-progress",
            ScanProgressEvent {
                source: source.clone(),
                status: "done".to_string(),
                found: found_count,
            },
        );
    }

    Ok(candidates)
}

fn scan_single_source(source: &ImportSource, user: &SteamUser) -> Vec<ImportCandidate> {
    match source {
        // Cross-platform (each importer handles OS internally)
        ImportSource::Gog => importers::gog::scan(user).unwrap_or_default(),
        ImportSource::Epic => importers::epic::scan(user).unwrap_or_default(),
        ImportSource::Origin => importers::origin::scan(user).unwrap_or_default(),
        ImportSource::UbisoftConnect => importers::ubisoft::scan(user).unwrap_or_default(),
        ImportSource::Itch => importers::itch::scan(user).unwrap_or_default(),

        // Windows-only
        #[cfg(windows)]
        ImportSource::Playnite => importers::playnite::scan(user).unwrap_or_default(),
        #[cfg(windows)]
        ImportSource::Amazon => importers::amazon::scan(user).unwrap_or_default(),
        #[cfg(windows)]
        ImportSource::GamePass => importers::gamepass::scan(user).unwrap_or_default(),

        // Unix-only
        #[cfg(unix)]
        ImportSource::Heroic => importers::heroic::scan(user).unwrap_or_default(),
        #[cfg(unix)]
        ImportSource::Legendary => importers::legendary::scan(user).unwrap_or_default(),
        #[cfg(unix)]
        ImportSource::Lutris => importers::lutris::scan(user).unwrap_or_default(),
        #[cfg(unix)]
        ImportSource::Flatpak => importers::flatpak::scan(user).unwrap_or_default(),
        #[cfg(unix)]
        ImportSource::Bottles => importers::bottles::scan(user).unwrap_or_default(),
        #[cfg(unix)]
        ImportSource::MiniGalaxy => importers::minigalaxy::scan(user).unwrap_or_default(),

        _ => vec![],
    }
}

fn enabled_sources(request: &ScanRequest) -> Vec<ImportSource> {
    if !request.include_sources.is_empty() {
        return request.include_sources.clone();
    }

    let mut sources = vec![
        ImportSource::Gog,
        ImportSource::Epic,
        ImportSource::Itch,
        ImportSource::Origin,
        ImportSource::UbisoftConnect,
    ];

    #[cfg(windows)]
    sources.extend([
        ImportSource::Playnite,
        ImportSource::Amazon,
        ImportSource::GamePass,
    ]);

    #[cfg(unix)]
    sources.extend([
        ImportSource::Heroic,
        ImportSource::Legendary,
        ImportSource::Lutris,
        ImportSource::Flatpak,
        ImportSource::Bottles,
        ImportSource::MiniGalaxy,
    ]);

    sources
}

pub fn shortcut_from_candidate(candidate: &ImportCandidate, grid_path: &Path) -> ShortcutEntry {
    let exe = candidate.effective_executable();
    ShortcutEntry {
        app_id: non_steam_app_id(&quote_path(exe), &candidate.name),
        app_name: candidate.name.clone(),
        exe: quote_path(exe),
        start_dir: quote_path(candidate.effective_start_dir()),
        icon: shortcut_icon(candidate, grid_path),
        shortcut_path: String::new(),
        launch_options: candidate
            .effective_launch_options()
            .unwrap_or("")
            .to_string(),
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
