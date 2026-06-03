use full_steam_ahead_lib::{
    error::{AppError, AppResult},
    models::{ApplyProgressEvent, ApplyRequest, ApplyStep, ChangeKind, Options, ScanProgressEvent, ScanRequest, UserSettings},
    paths,
    steam,
};
use std::{
    fs,
    io::{self, Write},
};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    eprint!("Detecting Steam... ");
    let install = steam::detect::detect_steam()?;
    eprintln!("{} user(s) found", install.users.len());

    if install.users.is_empty() {
        return Err(AppError::Message(
            "No Steam users found. Launch Steam at least once to create a user profile.".to_string(),
        ));
    }

    let user = if install.users.len() == 1 {
        install.users.into_iter().next().unwrap()
    } else {
        eprintln!("Multiple Steam users found:");
        for (i, u) in install.users.iter().enumerate() {
            let name = u.account_name.as_deref().unwrap_or("(unknown)");
            eprintln!("  [{}] {} ({})", i + 1, name, u.steam_id);
        }
        eprint!("Select user [1]: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let idx: usize = input.trim().parse().unwrap_or(1);
        install
            .users
            .into_iter()
            .nth(idx.saturating_sub(1))
            .ok_or_else(|| AppError::Message("Invalid user selection.".to_string()))?
    };

    let user_label = user.account_name.as_deref().unwrap_or("(unknown)");
    eprintln!("Using: {} ({})", user_label, user.steam_id);

    eprintln!("Scanning for games...");
    let scan_request = ScanRequest {
        user_steam_id: user.steam_id.clone(),
        include_sources: vec![],
    };

    let candidates = steam::sources::scan_sources_with_progress(
        |event: ScanProgressEvent| {
            if event.status == "done" && event.found > 0 {
                eprintln!("  {} — {} game(s)", event.source.display_name(), event.found);
            }
        },
        &user,
        &scan_request,
    )?;

    if candidates.is_empty() {
        eprintln!("No games found. Nothing to do.");
        return Ok(());
    }

    eprintln!("{} game(s) found total.", candidates.len());

    let settings = load_settings_quietly();
    let options = Options {
        stop_steam: settings.stop_steam,
        restart_steam: settings.restart_steam,
        replace_existing_artwork: false,
    };

    let backup_root = paths::app_data_dir()
        .join("backups")
        .join(chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string());

    let plan = steam::plan::build_preview_plan(&user, &candidates, &options, &backup_root)?;

    print_plan(&plan);

    if plan.changes.is_empty() {
        eprintln!("Everything is already up to date.");
        return Ok(());
    }

    eprint!("Apply these changes? [y/N] ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    if !input.trim().eq_ignore_ascii_case("y") {
        eprintln!("Aborted.");
        return Ok(());
    }

    eprintln!("Applying...");
    let request = ApplyRequest { plan, candidates, options };

    let result = steam::apply::apply_plan_with_progress(
        |event: ApplyProgressEvent| {
            let label = match &event.step {
                ApplyStep::StoppingSteam => "Stopping Steam".to_string(),
                ApplyStep::CreatingBackups => "Creating backups".to_string(),
                ApplyStep::ApplyingArtwork { game_name } => {
                    format!("Artwork: {}", game_name.as_deref().unwrap_or("..."))
                }
                ApplyStep::UpdatingShortcuts => "Updating shortcuts".to_string(),
                ApplyStep::UpdatingCollections => "Updating collections".to_string(),
                ApplyStep::RestartingSteam => "Restarting Steam".to_string(),
            };
            eprintln!("  [{}/{}] {}", event.current, event.total, label);
        },
        request,
    )?;

    eprintln!(
        "Done. {} change(s) applied, {} backup(s) created.",
        result.applied_changes.len(),
        result.backups_created.len()
    );

    Ok(())
}

fn print_plan(plan: &full_steam_ahead_lib::models::PreviewPlan) {
    let new_shortcuts: Vec<_> = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::AddShortcut))
        .collect();
    let updated: Vec<_> = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::UpdateShortcut))
        .collect();
    let artwork_count = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::WriteArtwork))
        .count();

    if !new_shortcuts.is_empty() {
        eprintln!("\nNew shortcuts ({}):", new_shortcuts.len());
        for c in &new_shortcuts {
            eprintln!("  + {}", c.game_name);
        }
    }
    if !updated.is_empty() {
        eprintln!("\nUpdated shortcuts ({}):", updated.len());
        for c in &updated {
            eprintln!("  ~ {}", c.game_name);
        }
    }
    if artwork_count > 0 {
        eprintln!("\nArtwork writes: {artwork_count}");
    }
    if !plan.backups.is_empty() {
        eprintln!("Backups to create: {}", plan.backups.len());
    }
    eprintln!();
}

fn load_settings_quietly() -> UserSettings {
    let path = paths::app_data_dir().join("settings.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}
