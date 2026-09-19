use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::{path::Path, sync::OnceLock};

pub(super) fn http_client() -> &'static reqwest::blocking::Client {
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
pub(super) struct StoreSearchResponse {
    pub(super) items: Vec<StoreSearchItem>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StoreSearchItem {
    pub(super) id: u32,
    pub(super) name: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct StoreItemsResponse {
    pub(super) response: StoreItemsBody,
}

#[derive(Debug, Deserialize)]
pub(super) struct StoreItemsBody {
    pub(super) store_items: Vec<StoreItem>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StoreItem {
    pub(super) assets: Option<StoreItemAssets>,
}

#[derive(Debug, Deserialize)]
pub(super) struct StoreItemAssets {
    pub(super) asset_url_format: String,
    pub(super) header: Option<String>,
    pub(super) header_2x: Option<String>,
    pub(super) library_capsule: Option<String>,
    pub(super) library_capsule_2x: Option<String>,
    pub(super) library_hero: Option<String>,
    pub(super) library_hero_2x: Option<String>,
    pub(super) logo: Option<String>,
    pub(super) logo_2x: Option<String>,
    pub(super) community_icon: Option<String>,
}

pub(super) fn find_steam_app_id(game_name: &str) -> Option<u32> {
    let url = reqwest::Url::parse_with_params(
        "https://store.steampowered.com/api/storesearch/",
        [("term", game_name), ("l", "en"), ("cc", "US")],
    )
    .ok()?
    .to_string();
    let response = http_client()
        .get(url.as_str())
        .send()
        .inspect_err(|error| tracing::warn!(%error, url, "Steam store search request failed"))
        .ok()?
        .error_for_status()
        .inspect_err(
            |error| tracing::warn!(%error, url, "Steam store search returned an error status"),
        )
        .ok()?;
    let search = response
        .json::<StoreSearchResponse>()
        .inspect_err(|error| tracing::warn!(%error, url, "Steam store search response was invalid"))
        .ok()?;
    search
        .items
        .into_iter()
        .min_by_key(|item| name_distance(&item.name, game_name))
        .map(|item| item.id)
}

pub(super) fn store_item_asset_specs(
    steam_app_id: u32,
) -> Option<Vec<(crate::models::ArtworkKind, String)>> {
    use crate::models::ArtworkKind;

    let request = serde_json::json!({
        "ids": [{ "appid": steam_app_id }],
        "context": { "country_code": "US" },
        "data_request": { "include_assets": true }
    });
    let url = reqwest::Url::parse_with_params(
        "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/",
        [("input_json", request.to_string())],
    )
    .ok()?
    .to_string();
    let item = http_client()
        .get(url.as_str())
        .send()
        .inspect_err(|error| tracing::warn!(%error, url, "Steam store item request failed"))
        .ok()?
        .error_for_status()
        .inspect_err(
            |error| tracing::warn!(%error, url, "Steam store item request returned an error status"),
        )
        .ok()?
        .json::<StoreItemsResponse>()
        .inspect_err(|error| tracing::warn!(%error, url, "Steam store item response was invalid"))
        .ok()?
        .response
        .store_items
        .into_iter()
        .next()?;
    let assets = item.assets?;
    let mut specs = Vec::new();

    super::push_store_asset(
        &mut specs,
        ArtworkKind::Header,
        &assets.asset_url_format,
        assets.header_2x.as_deref().or(assets.header.as_deref()),
    );
    super::push_store_asset(
        &mut specs,
        ArtworkKind::Capsule,
        &assets.asset_url_format,
        assets
            .library_capsule_2x
            .as_deref()
            .or(assets.library_capsule.as_deref()),
    );
    super::push_store_asset(
        &mut specs,
        ArtworkKind::Hero,
        &assets.asset_url_format,
        assets
            .library_hero_2x
            .as_deref()
            .or(assets.library_hero.as_deref()),
    );
    super::push_store_asset(
        &mut specs,
        ArtworkKind::Logo,
        &assets.asset_url_format,
        assets.logo_2x.as_deref().or(assets.logo.as_deref()),
    );

    super::fill_logo_fallback(&mut specs, &assets.asset_url_format, steam_app_id);

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

pub(super) fn community_icon_url(steam_app_id: u32) -> Option<String> {
    let url = format!("https://store.steampowered.com/app/{steam_app_id}/");
    let html = http_client()
        .get(url.as_str())
        .send()
        .inspect_err(|error| tracing::warn!(%error, url, "Steam community page request failed"))
        .ok()?
        .error_for_status()
        .inspect_err(
            |error| tracing::warn!(%error, url, "Steam community page returned an error status"),
        )
        .ok()?
        .text()
        .inspect_err(
            |error| tracing::warn!(%error, url, "Steam community page body was unreadable"),
        )
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

pub(super) fn reachable_url(url: &str) -> bool {
    http_client()
        .head(url)
        .send()
        .and_then(|response| response.error_for_status().map(|_| ()))
        .inspect_err(|error| tracing::debug!(%error, url, "Artwork URL not reachable"))
        .is_ok()
}

/// Artwork only ever comes from these hosts.
/// `download_asset`'s `url` argument can come from a webview-round-tripped
/// plan or a SteamGridDB API response. Both must be checked against this list.
const ALLOWED_ARTWORK_HOSTS: &[&str] = &[
    "cdn.cloudflare.steamstatic.com",
    "shared.cloudflare.steamstatic.com",
    "shared.steamstatic.com",
    "cdn2.steamgriddb.com",
];

fn is_allowed_artwork_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|parsed| {
        parsed.scheme() == "https"
            && parsed
                .host_str()
                .is_some_and(|host| ALLOWED_ARTWORK_HOSTS.contains(&host))
    })
}

pub(super) fn download_asset(url: &str, target: &Path) -> AppResult<()> {
    use crate::error::io_context;
    use std::fs;

    if !is_allowed_artwork_url(url) {
        return Err(AppError::Message(format!(
            "refusing to download artwork from an untrusted host: {url}"
        )));
    }

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

#[cfg(test)]
mod download_asset_tests {
    use super::is_allowed_artwork_url;

    #[test]
    fn allows_known_steam_cdn_host() {
        assert!(is_allowed_artwork_url(
            "https://shared.cloudflare.steamstatic.com/store_item_assets/foo.jpg"
        ));
    }

    #[test]
    fn allows_known_steamgriddb_cdn_host() {
        assert!(is_allowed_artwork_url(
            "https://cdn2.steamgriddb.com/grid/abc.png"
        ));
    }

    #[test]
    fn rejects_unknown_host() {
        assert!(!is_allowed_artwork_url("https://attacker.example/x"));
    }

    #[test]
    fn rejects_non_https_scheme() {
        assert!(!is_allowed_artwork_url(
            "http://shared.cloudflare.steamstatic.com/store_item_assets/foo.jpg"
        ));
    }

    #[test]
    fn rejects_malformed_url() {
        assert!(!is_allowed_artwork_url("not a url"));
    }

    #[test]
    fn rejects_loopback_and_link_local_targets() {
        assert!(!is_allowed_artwork_url("http://127.0.0.1:9999/internal"));
        assert!(!is_allowed_artwork_url(
            "http://169.254.169.254/latest/meta-data/"
        ));
    }
}

pub(super) fn name_distance(left: &str, right: &str) -> usize {
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
