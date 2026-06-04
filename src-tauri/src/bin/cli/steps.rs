use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use full_steam_ahead_lib::{
    error::{AppError, AppResult},
    models::{
        ApplyProgressEvent, ApplyRequest, ApplyResult, ApplyStep, ImportCandidate, Options,
        PreviewPlan, ScanProgressEvent, ScanRequest, SteamUser,
    },
    paths, steam,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::{settings, ui};

pub fn detect_and_select_user(theme: &ColorfulTheme) -> AppResult<SteamUser> {
    let sp = ui::make_spinner("Detecting Steam...");
    let install = steam::detect::detect_steam().inspect_err(|_| sp.finish_and_clear())?;
    sp.finish_and_clear();
    println!(
        "{} Steam found — {} user(s)",
        style("✓").green().bold(),
        install.users.len()
    );

    if install.users.is_empty() {
        return Err(AppError::Message(
            "No Steam users found. Launch Steam at least once.".to_string(),
        ));
    }

    let user = if install.users.len() == 1 {
        let u = install.users.into_iter().next().unwrap();
        println!(
            "{} {}",
            style("→").blue(),
            style(u.account_name.as_deref().unwrap_or(&u.steam_id)).bold()
        );
        u
    } else {
        println!();
        let labels: Vec<String> = install
            .users
            .iter()
            .map(|u| {
                format!(
                    "{}  ({})",
                    u.account_name.as_deref().unwrap_or("(unknown)"),
                    u.steam_id
                )
            })
            .collect();
        let idx = Select::with_theme(theme)
            .with_prompt("Select Steam user")
            .items(&labels)
            .default(0)
            .interact()
            .map_err(|e| AppError::Message(e.to_string()))?;
        install.users.into_iter().nth(idx).unwrap()
    };

    println!();
    Ok(user)
}

pub fn scan_sources(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let sp = ui::make_spinner("Scanning launchers...");
    let candidates = steam::sources::scan_sources_with_progress(
        |event: ScanProgressEvent| {
            if event.status == "scanning" {
                sp.set_message(format!("Scanning {}...", event.source.display_name()));
            } else if event.found > 0 {
                sp.println(format!(
                    "  {} {}  {}",
                    style("◆").cyan(),
                    event.source.display_name(),
                    style(format!("{} game(s)", event.found)).dim()
                ));
            }
        },
        user,
        &ScanRequest {
            user_steam_id: user.steam_id.clone(),
            include_sources: vec![],
        },
    )
    .inspect_err(|_| sp.finish_and_clear())?;
    sp.finish_and_clear();

    if !candidates.is_empty() {
        println!(
            "\n{} {} game(s) found\n",
            style("✓").green().bold(),
            style(candidates.len()).bold()
        );
    }

    Ok(candidates)
}

pub fn select_games(
    theme: &ColorfulTheme,
    candidates: Vec<ImportCandidate>,
) -> AppResult<Vec<ImportCandidate>> {
    let labels: Vec<String> = candidates
        .iter()
        .map(|c| format!("{}  [{}]", c.name, c.source.display_name()))
        .collect();
    let defaults = vec![true; candidates.len()];

    let selected_indices = MultiSelect::with_theme(theme)
        .with_prompt("Select games to import  (space to toggle, enter to confirm)")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .map_err(|e| AppError::Message(e.to_string()))?;

    println!();

    Ok(selected_indices
        .into_iter()
        .map(|i| candidates[i].clone())
        .collect())
}

pub fn build_plan(
    user: &SteamUser,
    selected: &[ImportCandidate],
) -> AppResult<(PreviewPlan, Options)> {
    let settings = settings::load_quietly();
    let options = Options {
        stop_steam: settings.stop_steam,
        restart_steam: settings.restart_steam,
        replace_existing_artwork: false,
    };
    let backup_root = paths::app_data_dir()
        .join("backups")
        .join(chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string());

    let sp = ui::make_spinner("Building plan...");
    let plan = steam::plan::build_preview_plan(user, selected, &options, &backup_root)
        .inspect_err(|_| sp.finish_and_clear())?;
    sp.finish_and_clear();

    Ok((plan, options))
}

pub fn confirm_apply(theme: &ColorfulTheme) -> AppResult<bool> {
    Confirm::with_theme(theme)
        .with_prompt("Apply these changes?")
        .default(false)
        .interact()
        .map_err(|e| AppError::Message(e.to_string()))
}

pub fn apply_changes(
    plan: PreviewPlan,
    selected: Vec<ImportCandidate>,
    options: Options,
) -> AppResult<ApplyResult> {
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan} [{bar:40.cyan/237}] {pos}/{len}  {msg}",
        )
        .unwrap()
        .progress_chars("█░░")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));

    let result = steam::apply::apply_plan_with_progress(
        |event: ApplyProgressEvent| {
            pb.set_length(event.total as u64);
            pb.set_position(event.current as u64);
            pb.set_message(match &event.step {
                ApplyStep::StoppingSteam => "Stopping Steam".to_string(),
                ApplyStep::CreatingBackups => "Creating backups".to_string(),
                ApplyStep::ApplyingArtwork { game_name } => {
                    format!("Artwork: {}", game_name.as_deref().unwrap_or("..."))
                }
                ApplyStep::UpdatingShortcuts => "Updating shortcuts".to_string(),
                ApplyStep::UpdatingCollections => "Updating collections".to_string(),
                ApplyStep::RestartingSteam => "Restarting Steam".to_string(),
            });
        },
        ApplyRequest {
            plan,
            candidates: selected,
            options,
        },
    )
    .inspect_err(|_| pb.finish_and_clear())?;

    pb.finish_and_clear();
    Ok(result)
}
