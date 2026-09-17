use super::fetch::http_client;
use crate::{
    error::{AppError, AppResult},
    models::{ArtworkKind, SteamGridDbGame, SteamGridDbImage},
};
use serde::Deserialize;

const BASE_URL: &str = "https://www.steamgriddb.com/api/v2";

pub fn search_games(api_key: &str, query: &str) -> AppResult<Vec<SteamGridDbGame>> {
    let response: ApiResponse<RawGame> = request(
        api_key,
        &endpoint_url(&["search", "autocomplete", query], &[])?,
    )?;
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
    allow_nsfw: bool,
) -> AppResult<Vec<SteamGridDbImage>> {
    let (endpoint, dimensions) = match kind {
        ArtworkKind::Header => ("grids", Some("460x215,920x430")),
        ArtworkKind::Capsule => ("grids", Some("600x900,342x482")),
        ArtworkKind::Hero => ("heroes", None),
        ArtworkKind::Logo => ("logos", None),
        ArtworkKind::Icon => ("icons", None),
    };

    let mut params = Vec::new();
    if let Some(dimensions) = dimensions {
        params.push(("dimensions", dimensions));
    }
    if allow_nsfw {
        params.push(("nsfw", "any"));
    }
    let url = endpoint_url(&[endpoint, "game", &game_id.to_string()], &params)?;

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

/// Builds an API URL, percent-encoding each path segment and query parameter.
fn endpoint_url(segments: &[&str], params: &[(&str, &str)]) -> AppResult<String> {
    let invalid = || AppError::Message("Invalid SteamGridDB URL".to_string());
    let mut url = reqwest::Url::parse(BASE_URL).map_err(|_| invalid())?;
    url.path_segments_mut()
        .map_err(|()| invalid())?
        .extend(segments);
    if !params.is_empty() {
        url.query_pairs_mut().extend_pairs(params);
    }
    Ok(url.into())
}

fn request<T: for<'de> Deserialize<'de>>(api_key: &str, url: &str) -> AppResult<ApiResponse<T>> {
    tracing::debug!(url, "SteamGridDB request");

    let response = http_client()
        .get(url)
        .bearer_auth(api_key)
        .send()
        .map_err(|error| AppError::Message(format!("SteamGridDB request failed: {error}")))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Message(
            "SteamGridDB rejected the API key. Check it in Settings.".to_string(),
        ));
    }

    let body = response.text().map_err(|error| {
        AppError::Message(format!("SteamGridDB response could not be read: {error}"))
    })?;

    if !status.is_success() {
        tracing::warn!(%status, body, "SteamGridDB request failed");
        return Err(AppError::Message(format!(
            "SteamGridDB request failed ({status}): {body}"
        )));
    }

    serde_json::from_str::<ApiResponse<T>>(&body).map_err(|error| {
        tracing::warn!(%error, body, "SteamGridDB response was not the expected shape");
        AppError::Message(format!("SteamGridDB response was invalid: {error}"))
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_terms_are_encoded_as_one_path_segment() {
        assert_eq!(
            endpoint_url(&["search", "autocomplete", "Half-Life 2: Ep/One +?#"], &[]).unwrap(),
            "https://www.steamgriddb.com/api/v2/search/autocomplete/Half-Life%202:%20Ep%2FOne%20+%3F%23"
        );
    }

    #[test]
    fn query_parameters_are_appended() {
        assert_eq!(
            endpoint_url(
                &["grids", "game", "42"],
                &[("dimensions", "460x215,920x430"), ("nsfw", "any")]
            )
            .unwrap(),
            "https://www.steamgriddb.com/api/v2/grids/game/42?dimensions=460x215%2C920x430&nsfw=any"
        );
        assert_eq!(
            endpoint_url(&["heroes", "game", "42"], &[]).unwrap(),
            "https://www.steamgriddb.com/api/v2/heroes/game/42"
        );
    }
}
