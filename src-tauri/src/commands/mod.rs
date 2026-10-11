mod app;
mod artwork;
mod backups;
mod debug;
mod settings;
mod steam;

pub use app::*;
pub use artwork::*;
pub use backups::*;
pub use debug::*;
pub use settings::*;
pub use steam::*;

use crate::error::{AppError, CommandError};

/// Runs slow work off the main thread. On Linux that thread also draws the window.
async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, CommandError> + Send + 'static,
) -> Result<T, CommandError> {
    let span = tracing::Span::current();
    tauri::async_runtime::spawn_blocking(move || span.in_scope(work))
        .await
        .map_err(|error| CommandError::from(AppError::Message(error.to_string())))?
}
