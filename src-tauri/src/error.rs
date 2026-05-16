use serde::Serialize;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Steam could not be found on this Windows account.")]
    SteamNotFound,
    #[error("The requested Steam user was not found: {0}")]
    UserNotFound(String),
    #[error("The file is not a valid Steam shortcuts.vdf file: {0}")]
    InvalidShortcuts(String),
    #[error("A filesystem operation failed for {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("A JSON operation failed for {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        let span_trace = tracing_error::SpanTrace::capture();
        tracing::error!(error = %value, span_trace = %span_trace, "Command failed");
        Self {
            message: value.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn io_context(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> AppError {
    let path = path.into();
    move |source| AppError::Io { path, source }
}
