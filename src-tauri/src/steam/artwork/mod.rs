mod fetch;
pub mod steamgriddb;

use crate::{
    error::{io_context, AppResult},
    models::{ArtworkAsset, ArtworkKind, ArtworkMode, ArtworkPlan, ArtworkSource, ImportCandidate},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct ArtworkSkip {
    pub change_id: String,
}

pub fn steam_preferred_plan(
    grid_path: &Path,
    shortcut_app_id: u32,
    game_name: &str,
) -> (Option<u32>, ArtworkPlan) {
    let existing = existing_assets(grid_path, shortcut_app_id);
    let Some(steam_app_id) = fetch::find_steam_app_id(game_name) else {
        return (
            None,
            ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing,
                proposed: Vec::new(),
            },
        );
    };

    let proposed = official_assets(steam_app_id, &existing);

    (
        Some(steam_app_id),
        ArtworkPlan {
            mode: ArtworkMode::OfficialSteamPreferred,
            existing,
            proposed,
        },
    )
}

pub fn preserve_existing_plan(grid_path: &Path, app_id: u32) -> ArtworkPlan {
    ArtworkPlan {
        mode: ArtworkMode::PreserveExisting,
        existing: existing_assets(grid_path, app_id),
        proposed: Vec::new(),
    }
}

pub fn apply_candidate_artwork(
    grid_path: &Path,
    candidate: &ImportCandidate,
) -> AppResult<Vec<ArtworkSkip>> {
    let shortcut_app_id = crate::steam::non_steam_app_id(
        &format!("\"{}\"", candidate.executable_path.display()),
        &candidate.name,
    );

    let mut skipped = Vec::new();
    for asset in selected_artwork_assets(candidate) {
        let target = target_path(grid_path, shortcut_app_id, &asset.kind, &asset.path_or_url);

        remove_stale_variants(
            grid_path,
            &artwork_stem(&asset.kind, shortcut_app_id),
            &target,
        );

        match asset.source {
            ArtworkSource::OfficialSteam | ArtworkSource::SteamGridDb => {
                tracing::debug!(kind = ?asset.kind, game = %candidate.name, url = %asset.path_or_url, "Downloading artwork");
                if let Err(error) = fetch::download_asset(&asset.path_or_url, &target) {
                    tracing::warn!(kind = ?asset.kind, game = %candidate.name, %error, "Artwork download failed");
                    skipped.push(ArtworkSkip {
                        change_id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
                    });
                }
            }
            ArtworkSource::LocalFile => {
                let source = Path::new(&asset.path_or_url);
                if let Err(error) = fs::copy(source, &target).map_err(io_context(&target)) {
                    tracing::warn!(kind = ?asset.kind, game = %candidate.name, %error, "Artwork copy failed");
                    skipped.push(ArtworkSkip {
                        change_id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
                    });
                }
            }
            ArtworkSource::ExistingCustom => {}
            ArtworkSource::Missing => {
                if target.exists() {
                    if let Err(error) = fs::remove_file(&target).map_err(io_context(&target)) {
                        tracing::warn!(kind = ?asset.kind, game = %candidate.name, %error, "Artwork delete failed");
                        skipped.push(ArtworkSkip {
                            change_id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
                        });
                    }
                }
            }
        }
    }

    Ok(skipped)
}

pub fn target_path(
    grid_path: &Path,
    app_id: u32,
    kind: &ArtworkKind,
    source_path_or_url: &str,
) -> PathBuf {
    let extension = extension_for(kind, source_path_or_url);
    let stem = artwork_stem(kind, app_id);
    grid_path.join(format!("{stem}.{extension}"))
}

fn artwork_stem(kind: &ArtworkKind, app_id: u32) -> String {
    match kind {
        ArtworkKind::Header => app_id.to_string(),
        ArtworkKind::Capsule => format!("{app_id}p"),
        ArtworkKind::Hero => format!("{app_id}_hero"),
        ArtworkKind::Logo => format!("{app_id}_logo"),
        ArtworkKind::Icon => format!("{app_id}_icon"),
    }
}

fn remove_stale_variants(grid_path: &Path, stem: &str, keep: &Path) {
    let Ok(entries) = fs::read_dir(grid_path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == keep {
            continue;
        }
        if path.file_stem().and_then(|value| value.to_str()) != Some(stem) {
            continue;
        }
        if let Err(error) = fs::remove_file(&path) {
            tracing::warn!(path = %path.display(), %error, "Failed to remove stale artwork variant");
        }
    }
}

pub fn selected_artwork_assets(candidate: &ImportCandidate) -> Vec<ArtworkAsset> {
    [
        ArtworkKind::Header,
        ArtworkKind::Capsule,
        ArtworkKind::Hero,
        ArtworkKind::Logo,
        ArtworkKind::Icon,
    ]
    .into_iter()
    .filter_map(|kind| {
        let marked_missing = candidate
            .artwork
            .proposed
            .iter()
            .any(|asset| asset.kind == kind && asset.source == ArtworkSource::Missing);
        if marked_missing {
            let existing = candidate
                .artwork
                .existing
                .iter()
                .find(|asset| asset.kind == kind);
            return Some(ArtworkAsset {
                kind,
                path_or_url: existing.map_or_else(String::new, |asset| asset.path_or_url.clone()),
                source: ArtworkSource::Missing,
                will_replace_existing: existing.is_some(),
            });
        }

        candidate
            .artwork
            .proposed
            .iter()
            .find(|asset| asset.kind == kind && asset.source == ArtworkSource::LocalFile)
            .or_else(|| {
                candidate
                    .artwork
                    .proposed
                    .iter()
                    .find(|asset| asset.kind == kind && asset.source == ArtworkSource::SteamGridDb)
            })
            .or_else(|| {
                candidate.artwork.proposed.iter().find(|asset| {
                    asset.kind == kind && asset.source == ArtworkSource::OfficialSteam
                })
            })
            .or_else(|| {
                candidate
                    .artwork
                    .proposed
                    .iter()
                    .find(|asset| asset.kind == kind)
            })
            .cloned()
    })
    .collect()
}

fn existing_assets(grid_path: &Path, app_id: u32) -> Vec<ArtworkAsset> {
    let prefixes = [
        ArtworkKind::Header,
        ArtworkKind::Capsule,
        ArtworkKind::Hero,
        ArtworkKind::Logo,
        ArtworkKind::Icon,
    ]
    .map(|kind| (kind.clone(), artwork_stem(&kind, app_id)));

    let mut existing = Vec::new();
    if let Ok(entries) = fs::read_dir(grid_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };

            for (kind, prefix) in &prefixes {
                if file_stem == prefix {
                    existing.push(ArtworkAsset {
                        kind: kind.clone(),
                        path_or_url: path.display().to_string(),
                        source: ArtworkSource::ExistingCustom,
                        will_replace_existing: false,
                    });
                }
            }
        }
    }
    existing
}

fn official_assets(steam_app_id: u32, existing: &[ArtworkAsset]) -> Vec<ArtworkAsset> {
    official_asset_specs(steam_app_id)
        .into_iter()
        .map(|(kind, url)| {
            let will_replace_existing = existing.iter().any(|asset| asset.kind == kind);
            ArtworkAsset {
                kind,
                path_or_url: url,
                source: ArtworkSource::OfficialSteam,
                will_replace_existing,
            }
        })
        .collect()
}

fn official_asset_specs(steam_app_id: u32) -> Vec<(ArtworkKind, String)> {
    if let Some(specs) = fetch::store_item_asset_specs(steam_app_id) {
        return specs;
    }

    let base = format!(
        "https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/{steam_app_id}"
    );
    let mut specs = vec![
        (ArtworkKind::Header, format!("{base}/header.jpg")),
        (ArtworkKind::Capsule, format!("{base}/library_600x900.jpg")),
        (ArtworkKind::Hero, format!("{base}/library_hero.jpg")),
        (ArtworkKind::Logo, format!("{base}/logo.png")),
    ];

    if let Some(icon_url) = fetch::community_icon_url(steam_app_id) {
        specs.push((ArtworkKind::Icon, icon_url));
    }

    specs
}

pub(super) fn push_store_asset(
    specs: &mut Vec<(ArtworkKind, String)>,
    kind: ArtworkKind,
    asset_url_format: &str,
    filename: Option<&str>,
) {
    if let Some(filename) = filename {
        specs.push((kind, store_asset_url(asset_url_format, filename)));
    }
}

pub(super) fn fill_logo_fallback(
    specs: &mut Vec<(ArtworkKind, String)>,
    asset_url_format: &str,
    steam_app_id: u32,
) {
    if specs.iter().any(|(kind, _)| *kind == ArtworkKind::Logo) {
        return;
    }
    push_store_asset(
        specs,
        ArtworkKind::Logo,
        asset_url_format,
        known_library_logo_2x(steam_app_id),
    );
    if specs.iter().any(|(kind, _)| *kind == ArtworkKind::Logo) {
        return;
    }
    for filename in ["logo_2x.png", "logo.png"] {
        push_reachable_store_asset(specs, ArtworkKind::Logo, asset_url_format, filename);
        if specs.iter().any(|(kind, _)| *kind == ArtworkKind::Logo) {
            return;
        }
    }
}

fn known_library_logo_2x(steam_app_id: u32) -> Option<&'static str> {
    match steam_app_id {
        3_089_420 => Some("331e53ee4e0e2dea265f3da1226c9de4dc05f72c/logo_2x.png"),
        _ => None,
    }
}

fn push_reachable_store_asset(
    specs: &mut Vec<(ArtworkKind, String)>,
    kind: ArtworkKind,
    asset_url_format: &str,
    filename: &str,
) {
    let url = store_asset_url(asset_url_format, filename);
    if fetch::reachable_url(&url) {
        specs.push((kind, url));
    }
}

pub(super) fn store_asset_url(asset_url_format: &str, filename: &str) -> String {
    format!(
        "https://shared.steamstatic.com/store_item_assets/{}",
        asset_url_format.replace("${FILENAME}", filename)
    )
}

fn extension_for(kind: &ArtworkKind, source_path_or_url: &str) -> &'static str {
    let path_without_query = source_path_or_url
        .split_once('?')
        .map(|(path, _query)| path)
        .unwrap_or(source_path_or_url);
    let path_without_query = path_without_query
        .split_once('#')
        .map(|(path, _fragment)| path)
        .unwrap_or(path_without_query);

    if let Some(extension) = Path::new(path_without_query)
        .extension()
        .and_then(|value| value.to_str())
    {
        if extension.eq_ignore_ascii_case("png") {
            return "png";
        }
        if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
            return "jpg";
        }
    }

    match kind {
        ArtworkKind::Logo | ArtworkKind::Icon => "png",
        ArtworkKind::Header | ArtworkKind::Capsule | ArtworkKind::Hero => "jpg",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArtworkMode, ArtworkPlan, ImportSource};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TmpDir(PathBuf);

    impl TmpDir {
        fn new() -> Self {
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("fsa_artwork_test_{}_{}", std::process::id(), n));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    // remove_stale_variants

    #[test]
    fn remove_stale_variants_deletes_other_extension() {
        let dir = TmpDir::new();
        let stale = dir.path().join("12345.jpg");
        let keep = dir.path().join("12345.png");
        fs::write(&stale, b"old official jpg").unwrap();
        fs::write(&keep, b"new steamgriddb png").unwrap();

        remove_stale_variants(dir.path(), "12345", &keep);

        assert!(!stale.exists());
        assert!(keep.exists());
    }

    #[test]
    fn remove_stale_variants_ignores_other_stems() {
        let dir = TmpDir::new();
        let unrelated = dir.path().join("12345p.jpg");
        let keep = dir.path().join("12345.png");
        fs::write(&unrelated, b"capsule, different stem").unwrap();
        fs::write(&keep, b"header png").unwrap();

        remove_stale_variants(dir.path(), "12345", &keep);

        assert!(unrelated.exists());
        assert!(keep.exists());
    }

    #[test]
    fn remove_stale_variants_no_op_when_nothing_stale() {
        let dir = TmpDir::new();
        let keep = dir.path().join("12345.jpg");
        fs::write(&keep, b"only file").unwrap();

        remove_stale_variants(dir.path(), "12345", &keep);

        assert!(keep.exists());
    }

    // target_path

    #[test]
    fn target_path_header_uses_bare_id() {
        let path = target_path(
            Path::new("/grid"),
            12345,
            &ArtworkKind::Header,
            "header.jpg",
        );
        assert_eq!(path, PathBuf::from("/grid/12345.jpg"));
    }

    #[test]
    fn target_path_capsule_appends_p() {
        let path = target_path(Path::new("/grid"), 12345, &ArtworkKind::Capsule, "img.jpg");
        assert_eq!(path, PathBuf::from("/grid/12345p.jpg"));
    }

    #[test]
    fn target_path_hero_appends_hero() {
        let path = target_path(Path::new("/grid"), 12345, &ArtworkKind::Hero, "img.jpg");
        assert_eq!(path, PathBuf::from("/grid/12345_hero.jpg"));
    }

    #[test]
    fn target_path_logo_appends_logo() {
        let path = target_path(Path::new("/grid"), 12345, &ArtworkKind::Logo, "img.png");
        assert_eq!(path, PathBuf::from("/grid/12345_logo.png"));
    }

    #[test]
    fn target_path_icon_appends_icon() {
        let path = target_path(Path::new("/grid"), 12345, &ArtworkKind::Icon, "img.png");
        assert_eq!(path, PathBuf::from("/grid/12345_icon.png"));
    }

    // extension_for

    #[test]
    fn extension_detects_jpg_from_url() {
        assert_eq!(
            extension_for(&ArtworkKind::Header, "https://example.com/img.jpg"),
            "jpg"
        );
    }

    #[test]
    fn extension_detects_png_from_url() {
        assert_eq!(
            extension_for(&ArtworkKind::Header, "https://example.com/img.png"),
            "png"
        );
    }

    #[test]
    fn extension_strips_query_string() {
        assert_eq!(
            extension_for(&ArtworkKind::Header, "https://example.com/img.jpg?v=1"),
            "jpg"
        );
    }

    #[test]
    fn extension_strips_fragment() {
        assert_eq!(
            extension_for(&ArtworkKind::Hero, "https://example.com/img.png#section"),
            "png"
        );
    }

    #[test]
    fn extension_jpeg_normalises_to_jpg() {
        assert_eq!(extension_for(&ArtworkKind::Capsule, "img.JPEG"), "jpg");
    }

    #[test]
    fn extension_falls_back_to_png_for_logo() {
        assert_eq!(
            extension_for(&ArtworkKind::Logo, "https://example.com/img"),
            "png"
        );
    }

    #[test]
    fn extension_falls_back_to_jpg_for_header() {
        assert_eq!(
            extension_for(&ArtworkKind::Header, "https://example.com/img"),
            "jpg"
        );
    }

    // selected_artwork_assets

    fn make_asset(kind: ArtworkKind, source: ArtworkSource) -> ArtworkAsset {
        ArtworkAsset {
            kind,
            path_or_url: "url".to_string(),
            source,
            will_replace_existing: false,
        }
    }

    fn make_candidate(proposed: Vec<ArtworkAsset>) -> ImportCandidate {
        ImportCandidate {
            id: "test".to_string(),
            source: ImportSource::Manual,
            name: "Game".to_string(),
            executable_path: PathBuf::from("game.exe"),
            start_dir: PathBuf::from("."),
            launch_options: None,
            existing_app_id: None,
            matched_steam_app_id: None,
            tags: vec![],
            artwork: ArtworkPlan {
                mode: ArtworkMode::PreserveExisting,
                existing: vec![],
                proposed,
            },
            url_scheme: None,
            launcher_path: None,
            use_launcher_url: false,
            needs_proton: false,
        }
    }

    #[test]
    fn selected_assets_empty_proposed_returns_empty() {
        let candidate = make_candidate(vec![]);
        assert!(selected_artwork_assets(&candidate).is_empty());
    }

    #[test]
    fn selected_assets_prefers_local_file_over_official_steam() {
        let candidate = make_candidate(vec![
            make_asset(ArtworkKind::Header, ArtworkSource::OfficialSteam),
            make_asset(ArtworkKind::Header, ArtworkSource::LocalFile),
        ]);
        let selected = selected_artwork_assets(&candidate);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].source, ArtworkSource::LocalFile);
    }

    #[test]
    fn selected_assets_one_per_kind() {
        let candidate = make_candidate(vec![
            make_asset(ArtworkKind::Header, ArtworkSource::OfficialSteam),
            make_asset(ArtworkKind::Capsule, ArtworkSource::OfficialSteam),
        ]);
        let selected = selected_artwork_assets(&candidate);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn selected_assets_deduplicates_same_kind() {
        let candidate = make_candidate(vec![
            make_asset(ArtworkKind::Hero, ArtworkSource::OfficialSteam),
            make_asset(ArtworkKind::Hero, ArtworkSource::SteamGridDb),
        ]);
        let selected = selected_artwork_assets(&candidate);
        assert_eq!(selected.len(), 1);
    }
}
