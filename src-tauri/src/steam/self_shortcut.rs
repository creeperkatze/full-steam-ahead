use crate::{
    error::{io_context, AppError, AppResult},
    importers::quote_path,
    models::{ArtworkKind, ShortcutEntry},
};
use std::{fs, path::Path};

const APP_NAME: &str = "Full Steam Ahead";

const HEADER: &[u8] = include_bytes!("../../assets/self/header.png");
const CAPSULE: &[u8] = include_bytes!("../../assets/self/capsule.png");
const HERO: &[u8] = include_bytes!("../../assets/self/hero.png");
const LOGO: &[u8] = include_bytes!("../../assets/self/logo.png");
const ICON: &[u8] = include_bytes!("../../assets/self/icon.png");

/// Builds the shortcut entry for Full Steam Ahead itself
pub fn build(grid_path: &Path) -> AppResult<ShortcutEntry> {
    let exe = std::env::current_exe().map_err(|source| {
        AppError::Message(format!(
            "Failed to determine the current executable: {source}"
        ))
    })?;
    let start_dir = exe.parent().unwrap_or(Path::new("."));
    let exe = quote_path(&exe);
    let app_id = super::non_steam_app_id(&exe, APP_NAME);

    write_artwork(grid_path, app_id)?;

    Ok(ShortcutEntry {
        app_id,
        app_name: APP_NAME.to_string(),
        exe,
        start_dir: quote_path(start_dir),
        icon: super::artwork::target_path(grid_path, app_id, &ArtworkKind::Icon, "icon.png")
            .display()
            .to_string(),
        ..ShortcutEntry::default()
    })
}

fn write_artwork(grid_path: &Path, app_id: u32) -> AppResult<()> {
    fs::create_dir_all(grid_path).map_err(io_context(grid_path))?;

    for (kind, filename, bytes) in [
        (ArtworkKind::Header, "header.png", HEADER),
        (ArtworkKind::Capsule, "capsule.png", CAPSULE),
        (ArtworkKind::Hero, "hero.png", HERO),
        (ArtworkKind::Logo, "logo.png", LOGO),
        (ArtworkKind::Icon, "icon.png", ICON),
    ] {
        let target = super::artwork::target_path(grid_path, app_id, &kind, filename);
        fs::write(&target, bytes).map_err(io_context(&target))?;
    }

    Ok(())
}
