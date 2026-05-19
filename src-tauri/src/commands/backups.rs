use crate::{backups, error::CommandError, models::BackupInfo};
use tracing::{info, instrument};

type CommandResult<T> = Result<T, CommandError>;

#[tauri::command]
#[instrument]
pub fn list_backups() -> CommandResult<Vec<BackupInfo>> {
    backups::list().map_err(Into::into)
}

#[tauri::command]
#[instrument]
pub fn restore_backup(backup_id: String) -> CommandResult<usize> {
    let restored = backups::restore_backup(&backup_id)?;
    info!(backup_id, restored, "Backup restored via command");
    Ok(restored)
}
