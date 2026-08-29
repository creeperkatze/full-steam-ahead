use crate::{
    error::CommandError,
    models::{ArtworkKind, SteamGridDbGame, SteamGridDbImage},
    steam,
};
use tracing::instrument;

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument(skip(api_key), err(Debug))]
pub fn steamgriddb_search(api_key: String, query: String) -> CommandResult<Vec<SteamGridDbGame>> {
    steam::artwork::steamgriddb::search_games(&api_key, &query).map_err(Into::into)
}

#[tauri::command]
#[instrument(skip(api_key), err(Debug))]
pub fn steamgriddb_images(
    api_key: String,
    game_id: u32,
    kind: ArtworkKind,
    allow_nsfw: bool,
) -> CommandResult<Vec<SteamGridDbImage>> {
    steam::artwork::steamgriddb::fetch_images(&api_key, game_id, &kind, allow_nsfw)
        .map_err(Into::into)
}
