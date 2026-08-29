use crate::{backups, error::CommandError, models::BackupInfo};
use tracing::instrument;

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
    Ok(restored)
}

#[tauri::command]
#[instrument]
pub fn delete_backup(backup_id: String) -> CommandResult<()> {
    backups::delete_backup(&backup_id)?;
    Ok(())
}

#[tauri::command]
#[instrument]
pub fn delete_all_backups() -> CommandResult<()> {
    backups::delete_all_backups()?;
    Ok(())
}
