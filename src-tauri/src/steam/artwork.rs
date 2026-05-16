use crate::{
    error::{io_context, AppError, AppResult},
    models::{ArtworkAsset, ArtworkKind, ArtworkMode, ArtworkPlan, ArtworkSource, ImportCandidate},
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent(concat!(
                "creeperkatze/full-steam-ahead/",
                env!("CARGO_PKG_VERSION"),
                " (contact@creeperkatze.dev)"
            ))
            .build()
            .expect("failed to build HTTP client")
    })
}

#[derive(Debug, Deserialize)]
struct StoreSearchResponse {
    items: Vec<StoreSearchItem>,
}

#[derive(Debug, Deserialize)]
struct StoreSearchItem {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct StoreItemsResponse {
    response: StoreItemsBody,
}

#[derive(Debug, Deserialize)]
struct StoreItemsBody {
    store_items: Vec<StoreItem>,
}

#[derive(Debug, Deserialize)]
struct StoreItem {
    assets: Option<StoreItemAssets>,
}

#[derive(Debug, Deserialize)]
struct StoreItemAssets {
    asset_url_format: String,
    header: Option<String>,
    header_2x: Option<String>,
    library_capsule: Option<String>,
    library_capsule_2x: Option<String>,
    library_hero: Option<String>,
    library_hero_2x: Option<String>,
    logo: Option<String>,
    logo_2x: Option<String>,
    community_icon: Option<String>,
}

pub fn steam_preferred_plan(
    grid_path: &Path,
    shortcut_app_id: u32,
    game_name: &str,
) -> (Option<u32>, ArtworkPlan) {
    let existing = existing_assets(grid_path, shortcut_app_id);
    let Some(steam_app_id) = find_steam_app_id(game_name) else {
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

pub struct ArtworkSkip {
    pub change_id: String,
}

pub fn apply_candidate_artwork(
    grid_path: &Path,
    candidate: &ImportCandidate,
    replace_existing: bool,
) -> AppResult<Vec<ArtworkSkip>> {
    let shortcut_app_id = crate::steam::non_steam_app_id(
        &format!("\"{}\"", candidate.executable_path.display()),
        &candidate.name,
    );

    let mut skipped = Vec::new();
    for asset in selected_artwork_assets(candidate) {
        let target = target_path(grid_path, shortcut_app_id, &asset.kind, &asset.path_or_url);
        if target.exists() && !replace_existing && asset.source != ArtworkSource::LocalFile {
            tracing::debug!(kind = ?asset.kind, game = %candidate.name, "Preserving existing artwork");
            skipped.push(ArtworkSkip {
                change_id: format!("artwork:{}:{}", candidate.id, asset.kind.slug()),
            });
            continue;
        }

        match asset.source {
            ArtworkSource::OfficialSteam | ArtworkSource::SteamGridDb => {
                tracing::debug!(kind = ?asset.kind, game = %candidate.name, url = %asset.path_or_url, "Downloading artwork");
                if let Err(error) = download_asset(&asset.path_or_url, &target) {
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
    let stem = match kind {
        ArtworkKind::Header => app_id.to_string(),
        ArtworkKind::Capsule => format!("{app_id}p"),
        ArtworkKind::Hero => format!("{app_id}_hero"),
        ArtworkKind::Logo => format!("{app_id}_logo"),
        ArtworkKind::Icon => format!("{app_id}_icon"),
    };
    grid_path.join(format!("{stem}.{extension}"))
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
        candidate
            .artwork
            .proposed
            .iter()
            .find(|asset| asset.kind == kind && asset.source == ArtworkSource::LocalFile)
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
        (ArtworkKind::Header, app_id.to_string()),
        (ArtworkKind::Capsule, format!("{app_id}p")),
        (ArtworkKind::Hero, format!("{app_id}_hero")),
        (ArtworkKind::Logo, format!("{app_id}_logo")),
        (ArtworkKind::Icon, format!("{app_id}_icon")),
    ];

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
    if let Some(specs) = store_item_asset_specs(steam_app_id) {
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

    if let Some(icon_url) = community_icon_url(steam_app_id) {
        specs.push((ArtworkKind::Icon, icon_url));
    }

    specs
}

fn store_item_asset_specs(steam_app_id: u32) -> Option<Vec<(ArtworkKind, String)>> {
    let request = serde_json::json!({
        "ids": [{ "appid": steam_app_id }],
        "context": { "country_code": "US" },
        "data_request": { "include_assets": true }
    });
    let url = format!(
        "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/?input_json={}",
        encode_query(&request.to_string())
    );
    let item = http_client()
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?
        .json::<StoreItemsResponse>()
        .ok()?
        .response
        .store_items
        .into_iter()
        .next()?;
    let assets = item.assets?;
    let mut specs = Vec::new();

    push_store_asset(
        &mut specs,
        ArtworkKind::Header,
        &assets.asset_url_format,
        assets.header_2x.as_deref().or(assets.header.as_deref()),
    );
    push_store_asset(
        &mut specs,
        ArtworkKind::Capsule,
        &assets.asset_url_format,
        assets
            .library_capsule_2x
            .as_deref()
            .or(assets.library_capsule.as_deref()),
    );
    push_store_asset(
        &mut specs,
        ArtworkKind::Hero,
        &assets.asset_url_format,
        assets
            .library_hero_2x
            .as_deref()
            .or(assets.library_hero.as_deref()),
    );
    push_store_asset(
        &mut specs,
        ArtworkKind::Logo,
        &assets.asset_url_format,
        assets.logo_2x.as_deref().or(assets.logo.as_deref()),
    );

    fill_logo_fallback(&mut specs, &assets.asset_url_format, steam_app_id);

    if let Some(icon_hash) = assets.community_icon.filter(|hash| {
        hash.len() == 40 && hash.chars().all(|character| character.is_ascii_hexdigit())
    }) {
        specs.push((
            ArtworkKind::Icon,
            format!(
                "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{steam_app_id}/{icon_hash}.jpg"
            ),
        ));
    }

    (!specs.is_empty()).then_some(specs)
}

fn known_library_logo_2x(steam_app_id: u32) -> Option<&'static str> {
    match steam_app_id {
        3_089_420 => Some("331e53ee4e0e2dea265f3da1226c9de4dc05f72c/logo_2x.png"),
        _ => None,
    }
}

fn fill_logo_fallback(
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

fn push_store_asset(
    specs: &mut Vec<(ArtworkKind, String)>,
    kind: ArtworkKind,
    asset_url_format: &str,
    filename: Option<&str>,
) {
    if let Some(filename) = filename {
        specs.push((kind, store_asset_url(asset_url_format, filename)));
    }
}

fn push_reachable_store_asset(
    specs: &mut Vec<(ArtworkKind, String)>,
    kind: ArtworkKind,
    asset_url_format: &str,
    filename: &str,
) {
    let url = store_asset_url(asset_url_format, filename);
    if reachable_url(&url) {
        specs.push((kind, url));
    }
}

fn store_asset_url(asset_url_format: &str, filename: &str) -> String {
    format!(
        "https://shared.steamstatic.com/store_item_assets/{}",
        asset_url_format.replace("${FILENAME}", filename)
    )
}

fn reachable_url(url: &str) -> bool {
    http_client()
        .head(url)
        .send()
        .and_then(|response| response.error_for_status().map(|_| ()))
        .is_ok()
}

fn community_icon_url(steam_app_id: u32) -> Option<String> {
    let url = format!("https://store.steampowered.com/app/{steam_app_id}/");
    let html = http_client()
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .ok()?;
    let marker = format!("steamcommunity/public/images/apps/{steam_app_id}/");
    let marker_start = html.find(&marker)?;
    let url_start = html[..marker_start].rfind("https://")?;
    let extension_end = html[marker_start..].find(".jpg")? + marker_start + ".jpg".len();
    let icon_url = html[url_start..extension_end].replace("\\/", "/");
    let hash_start = marker_start + marker.len();
    let hash = html[hash_start..].get(..40)?;

    hash.chars()
        .all(|character| character.is_ascii_hexdigit())
        .then_some(icon_url)
}

fn find_steam_app_id(game_name: &str) -> Option<u32> {
    let term = encode_query(game_name);
    let url = format!("https://store.steampowered.com/api/storesearch/?term={term}&l=en&cc=US");
    let response = http_client()
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    let search = response.json::<StoreSearchResponse>().ok()?;
    search
        .items
        .into_iter()
        .min_by_key(|item| name_distance(&item.name, game_name))
        .map(|item| item.id)
}

fn name_distance(left: &str, right: &str) -> usize {
    let left = normalize_name(left);
    let right = normalize_name(right);
    if left == right {
        return 0;
    }
    if left.contains(&right) || right.contains(&left) {
        return 1;
    }
    left.len().abs_diff(right.len()) + 10
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn encode_query(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => vec![byte as char],
            b' ' => vec!['+'],
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn download_asset(url: &str, target: &Path) -> AppResult<()> {
    let response = http_client()
        .get(url)
        .send()
        .map_err(|error| AppError::Message(format!("request failed for {url}: {error}")))?
        .error_for_status()
        .map_err(|error| AppError::Message(format!("request failed for {url}: {error}")))?;
    let bytes = response.bytes().map_err(|error| {
        AppError::Message(format!(
            "could not read artwork response for {url}: {error}"
        ))
    })?;

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }
    fs::write(target, bytes).map_err(io_context(target))
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
