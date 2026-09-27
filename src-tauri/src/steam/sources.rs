use crate::{
    error::AppResult,
    importers::{self, quote_path},
    models::{
        ArtworkKind, ArtworkSource, ImportCandidate, ImportSource, ScanProgressEvent, ScanRequest,
        Settings, ShortcutEntry, SteamUser,
    },
    steam::{artwork, candidate_app_id, shortcuts},
};
use std::path::Path;

pub fn scan_sources_with_progress(
    on_progress: impl Fn(ScanProgressEvent),
    user: &SteamUser,
    request: &ScanRequest,
    settings: &Settings,
) -> AppResult<Vec<ImportCandidate>> {
    let mut candidates = Vec::new();
    let enabled_sources = enabled_sources(request, settings);
    let existing_shortcuts = read_existing_shortcuts(user);

    for source in &enabled_sources {
        on_progress(ScanProgressEvent {
            source: source.clone(),
            status: "scanning".to_string(),
            found: 0,
        });

        let source_settings = settings.source_settings(source);
        let custom_path = source_settings.custom_path.as_deref().map(Path::new);
        let _span = tracing::info_span!(
            "scan",
            source = %source.display_name(),
            custom_path = custom_path.map(|p| p.display().to_string())
        )
        .entered();
        let mut found = scan_single_source(source, user, custom_path);
        for candidate in &mut found {
            prepare_candidate(candidate, user, &existing_shortcuts, settings, true);
        }
        let found_count = found.len();
        candidates.extend(found);

        if found_count == 0 {
            tracing::debug!("No games found");
        } else {
            tracing::info!(found = found_count, "Games found");
        }

        on_progress(ScanProgressEvent {
            source: source.clone(),
            status: "done".to_string(),
            found: found_count,
        });
    }

    Ok(candidates)
}

pub fn read_existing_shortcuts(user: &SteamUser) -> Vec<ShortcutEntry> {
    shortcuts::read_shortcuts(&user.shortcuts_path).unwrap_or_else(|error| {
        tracing::warn!(%error, "Existing shortcuts could not be read");
        Vec::new()
    })
}

/// Applies the user's settings and links the candidate to the shortcut it already has in Steam.
pub fn prepare_candidate(
    candidate: &mut ImportCandidate,
    user: &SteamUser,
    existing_shortcuts: &[ShortcutEntry],
    settings: &Settings,
    use_steam_name: bool,
) {
    candidate.apply_launcher_mode(settings.launcher_mode);

    let existing = shortcuts::find_existing(existing_shortcuts, candidate);
    if let Some(shortcut) = existing {
        candidate.existing_app_id = Some(shortcut.app_id);
        artwork::relink_existing(&mut candidate.artwork, &user.grid_path, shortcut.app_id);
    }

    // Searched by the launcher's name, which finds more than a name the user made up.
    artwork::apply_source_preference(
        &mut candidate.artwork,
        &candidate.name,
        settings.default_artwork_source,
        &settings.steam_grid_db,
    );

    // A rename in Steam is kept instead of being reverted on the next import.
    if let Some(shortcut) = existing.filter(|s| use_steam_name && s.app_name != candidate.name) {
        tracing::debug!(game = %candidate.name, steam_name = %shortcut.app_name, "Using the name from Steam");
        candidate.name = shortcut.app_name.clone();
    }
}

type ScanFn = fn(&SteamUser, Option<&Path>) -> AppResult<Vec<ImportCandidate>>;

/// Single source of truth for scan dispatch and `scannable_sources`.
fn importer_registry() -> Vec<(ImportSource, ScanFn)> {
    let mut registry: Vec<(ImportSource, ScanFn)> = vec![
        (ImportSource::Gog, importers::gog::scan),
        (ImportSource::Epic, importers::epic::scan),
        (ImportSource::Itch, importers::itch::scan),
        (ImportSource::EaApp, importers::ea::scan),
        (ImportSource::UbisoftConnect, importers::ubisoft::scan),
    ];

    #[cfg(windows)]
    registry.extend([
        (ImportSource::Playnite, importers::playnite::scan as ScanFn),
        (ImportSource::Amazon, importers::amazon::scan as ScanFn),
        (ImportSource::Xbox, importers::xbox::scan as ScanFn),
    ]);

    #[cfg(unix)]
    registry.extend([
        (ImportSource::Heroic, importers::heroic::scan as ScanFn),
        (
            ImportSource::Legendary,
            importers::legendary::scan as ScanFn,
        ),
        (ImportSource::Lutris, importers::lutris::scan as ScanFn),
        (ImportSource::Flatpak, importers::flatpak::scan as ScanFn),
        (ImportSource::Bottles, importers::bottles::scan as ScanFn),
        (
            ImportSource::MiniGalaxy,
            importers::minigalaxy::scan as ScanFn,
        ),
    ]);

    registry
}

fn scan_single_source(
    source: &ImportSource,
    user: &SteamUser,
    custom_path: Option<&Path>,
) -> Vec<ImportCandidate> {
    let Some((_, scan)) = importer_registry()
        .into_iter()
        .find(|(candidate, _)| candidate == source)
    else {
        tracing::warn!("Source is not available on this platform");
        return Vec::new();
    };
    scan(user, custom_path).unwrap_or_else(|error| {
        tracing::warn!(%error, "Scan failed");
        Vec::new()
    })
}

/// All launcher sources this build knows how to scan for, in OS-appropriate order.
pub fn scannable_sources() -> Vec<ImportSource> {
    importer_registry()
        .into_iter()
        .map(|(source, _)| source)
        .collect()
}

fn enabled_sources(request: &ScanRequest, settings: &Settings) -> Vec<ImportSource> {
    let sources = if !request.include_sources.is_empty() {
        request.include_sources.clone()
    } else {
        scannable_sources()
    };

    sources
        .into_iter()
        .filter(|source| settings.source_settings(source).enabled)
        .collect()
}

pub fn shortcut_from_candidate(candidate: &ImportCandidate, grid_path: &Path) -> ShortcutEntry {
    let exe = candidate.effective_executable();
    ShortcutEntry {
        app_id: candidate_app_id(candidate),
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
        ArtworkSource::ExistingCustom | ArtworkSource::Missing => {
            Path::new(&asset.path_or_url).to_path_buf()
        }
        ArtworkSource::OfficialSteam | ArtworkSource::SteamGridDb | ArtworkSource::LocalFile => {
            let app_id = candidate_app_id(candidate);
            artwork::target_path(grid_path, app_id, &ArtworkKind::Icon, &asset.path_or_url)
        }
    };

    if icon_path.exists() {
        icon_path.display().to_string()
    } else {
        fallback
    }
}
