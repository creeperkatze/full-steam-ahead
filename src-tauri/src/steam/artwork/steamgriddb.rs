use super::fetch::{encode_query, http_client};
use crate::{
    error::{AppError, AppResult},
    models::{ArtworkKind, SteamGridDbGame, SteamGridDbImage},
};
use serde::Deserialize;

const BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

pub fn search_games(api_key: &str, query: &str) -> AppResult<Vec<SteamGridDbGame>> {
    let term = encode_query(query);
    let url = format!("{BASE_URL}/search/autocomplete/{term}");
    let response: ApiResponse<RawGame> = request(api_key, &url)?;
    Ok(response
        .data
        .into_iter()
        .map(|game| SteamGridDbGame {
            id: game.id,
            name: game.name,
        })
        .collect())
}

pub fn fetch_images(
    api_key: &str,
    game_id: u32,
    kind: &ArtworkKind,
) -> AppResult<Vec<SteamGridDbImage>> {
    let (endpoint, dimensions) = match kind {
        ArtworkKind::Header => ("grids", Some("460x215,920x430")),
        ArtworkKind::Capsule => ("grids", Some("600x900,342x482")),
        ArtworkKind::Hero => ("heroes", None),
        ArtworkKind::Logo => ("logos", None),
        ArtworkKind::Icon => ("icons", None),
    };

    let mut url = format!("{BASE_URL}/{endpoint}/game/{game_id}");
    if let Some(dimensions) = dimensions {
        url.push_str(&format!("?dimensions={dimensions}"));
    }

    let response: ApiResponse<RawImage> = request(api_key, &url)?;
    Ok(response
        .data
        .into_iter()
        .map(|image| SteamGridDbImage {
            id: image.id,
            url: image.url,
            thumbnail_url: image.thumb,
            width: image.width,
            height: image.height,
        })
        .collect())
}

fn request<T: for<'de> Deserialize<'de>>(api_key: &str, url: &str) -> AppResult<ApiResponse<T>> {
    let response = http_client()
        .get(url)
        .bearer_auth(api_key)
        .send()
        .map_err(|error| AppError::Message(format!("SteamGridDB request failed: {error}")))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Message(
            "SteamGridDB rejected the API key. Check it in Settings.".to_string(),
        ));
    }

    response
        .error_for_status()
        .map_err(|error| AppError::Message(format!("SteamGridDB request failed: {error}")))?
        .json::<ApiResponse<T>>()
        .map_err(|error| AppError::Message(format!("SteamGridDB response was invalid: {error}")))
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct RawGame {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawImage {
    id: u32,
    url: String,
    thumb: String,
    width: u32,
    height: u32,
}
