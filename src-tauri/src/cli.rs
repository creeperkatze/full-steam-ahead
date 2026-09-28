use crate::{
    error::{AppError, AppResult},
    importers::scan,
    models::{
        ApplyRequest, ApplyStep, ChangeKind, ImportSource, PlannedChange, ScanRequest, ScanStatus,
        SteamUser,
    },
    paths, steam,
};
use anstream::{eprintln, println};
use anstyle::{AnsiColor, RgbColor, Style};
use clap::{
    builder::{PossibleValuesParser, Styles},
    Parser, Subcommand,
};

mod logo;

const BOLD: Style = Style::new().bold();
const DIM: Style = Style::new().dimmed();
const ADDED: Style = AnsiColor::Green.on_default();
const UPDATED: Style = AnsiColor::Yellow.on_default();
const SUCCESS: Style = AnsiColor::Green.on_default().bold();
const ERROR: Style = AnsiColor::Red.on_default().bold();

const HELP_STYLES: Styles = Styles::styled()
    .header(RgbColor(26, 159, 255).on_default().bold())
    .usage(RgbColor(26, 159, 255).on_default().bold())
    .literal(Style::new().bold())
    .placeholder(DIM)
    .error(ERROR)
    .valid(ADDED)
    .invalid(UPDATED);

#[derive(Parser)]
#[command(version, about, styles = HELP_STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Import games into Steam with the saved settings, without opening a window.
    Import(ImportArgs),
}

#[derive(clap::Args)]
struct ImportArgs {
    /// Steam user to import for, as its userdata folder ID. Required when there are several.
    #[arg(long, value_name = "ID")]
    user: Option<String>,

    /// Only scan these sources, even ones turned off in the settings.
    #[arg(long = "source", value_name = "SOURCE", value_delimiter = ',', value_parser = source_parser())]
    sources: Vec<String>,

    /// Show what would change without writing anything.
    #[arg(long)]
    dry_run: bool,
}

/// Whether the app was started with a command instead of to open the window.
pub fn requested() -> bool {
    std::env::args_os().len() > 1
}

/// Runs the command given on the command line. Returns `None` when the window should open instead.
pub fn run() -> Option<i32> {
    if !requested() {
        return None;
    }
    attach_console();

    let Command::Import(args) = Cli::parse().command?;
    match import(&args) {
        Ok(()) => Some(0),
        Err(error) => {
            tracing::error!(%error, "Command line import failed");
            eprintln!("{ERROR}error:{ERROR:#} {error}");
            Some(1)
        }
    }
}

/// Release builds on Windows start without a console, so output would go nowhere.
#[cfg(windows)]
fn attach_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    // Fails harmlessly when there is no parent console or one is already attached.
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

#[cfg(not(windows))]
fn attach_console() {}

fn source_parser() -> PossibleValuesParser {
    PossibleValuesParser::new(
        scan::scannable_sources()
            .iter()
            .filter_map(ImportSource::settings_key),
    )
}

fn import(args: &ImportArgs) -> AppResult<()> {
    let mut settings =
        crate::commands::load_settings().map_err(|error| AppError::Message(error.message))?;
    let include_sources: Vec<ImportSource> = scan::scannable_sources()
        .into_iter()
        .filter(|source| {
            source
                .settings_key()
                .is_some_and(|key| args.sources.iter().any(|arg| arg == key))
        })
        .collect();
    for source in &include_sources {
        if let Some(key) = source.settings_key() {
            settings.sources.entry(key.to_string()).or_default().enabled = true;
        }
    }

    let user = select_user(args.user.as_deref())?;
    logo::print();
    println!("Scanning sources for {BOLD}{}{BOLD:#}", user_label(&user));
    let request = ScanRequest {
        user_steam_id: user.steam_id.clone(),
        include_sources,
    };
    let candidates = scan::scan_sources_with_progress(
        |event| {
            if matches!(event.status, ScanStatus::Done) {
                let count = if event.found == 0 { DIM } else { BOLD };
                println!(
                    "  {}  {count}{}{count:#}",
                    event.source.display_name(),
                    event.found
                );
            }
        },
        &user,
        &request,
        &settings,
    )?;

    let plan =
        steam::plan::build_preview_plan(&user, &candidates, &settings, &paths::new_backup_dir())?;
    tracing::info!(
        found = candidates.len(),
        changes = plan.changes.len(),
        dry_run = args.dry_run,
        "Command line import planned"
    );
    if plan.changes.is_empty() {
        println!("{SUCCESS}Steam is already up to date.{SUCCESS:#}");
        return Ok(());
    }
    print_changes(&plan.changes);
    if args.dry_run {
        println!("{DIM}Dry run, nothing was changed.{DIM:#}");
        return Ok(());
    }

    let restart_needed = plan.requires_steam_restart && !settings.restart_steam;
    let result = steam::apply::apply_plan_with_progress(
        |event| {
            println!(
                "{DIM}[{}/{}]{DIM:#} {}",
                event.current,
                event.total,
                step_label(&event.step)
            );
        },
        ApplyRequest {
            plan,
            candidates,
            options: settings,
        },
    )?;
    tracing::info!(
        applied = result.applied_changes.len(),
        backups = result.backups_created.len(),
        "Plan applied"
    );
    println!(
        "{SUCCESS}Done.{SUCCESS:#} Applied {} changes.",
        result.applied_changes.len()
    );
    if restart_needed {
        println!("Restart Steam to see them.");
    }
    Ok(())
}

fn select_user(steam_id: Option<&str>) -> AppResult<SteamUser> {
    if let Some(steam_id) = steam_id {
        return steam::detect::find_user(steam_id);
    }
    let mut users = steam::detect::detect_steam()?.users;
    match users.len() {
        0 => Err(AppError::Message(
            "Steam has no users yet. Sign in to Steam once, then try again.".to_string(),
        )),
        1 => Ok(users.remove(0)),
        _ => {
            let list: Vec<String> = users
                .iter()
                .map(|user| format!("  {}", user_label(user)))
                .collect();
            Err(AppError::Message(format!(
                "Steam has several users. Pick one with --user <ID>:\n{}",
                list.join("\n")
            )))
        }
    }
}

fn user_label(user: &SteamUser) -> String {
    match &user.account_name {
        Some(name) => format!("{} ({name})", user.steam_id),
        None => user.steam_id.clone(),
    }
}

fn print_changes(changes: &[PlannedChange]) {
    let mut artwork = 0;
    let mut collections = false;
    for change in changes {
        match change.kind {
            ChangeKind::AddShortcut => println!("  {ADDED}+{ADDED:#} {}", change.game_name),
            ChangeKind::UpdateShortcut => {
                println!("  {UPDATED}~{UPDATED:#} {}", change.game_name);
            }
            ChangeKind::WriteArtwork => artwork += 1,
            ChangeKind::UpdateCollections => collections = true,
        }
    }
    if artwork > 0 {
        println!("  {DIM}Write {artwork} artwork files{DIM:#}");
    }
    if collections {
        println!("  {DIM}Update collections{DIM:#}");
    }
}

fn step_label(step: &ApplyStep) -> String {
    match step {
        ApplyStep::StoppingSteam => "Stopping Steam".to_string(),
        ApplyStep::CreatingBackups => "Creating backups".to_string(),
        ApplyStep::ApplyingArtwork {
            game_name: Some(name),
        } => format!("Applying artwork for {name}"),
        ApplyStep::ApplyingArtwork { game_name: None } => "Applying artwork".to_string(),
        ApplyStep::UpdatingShortcuts => "Updating shortcuts".to_string(),
        ApplyStep::UpdatingCollections => "Updating collections".to_string(),
        ApplyStep::RestartingSteam => "Restarting Steam".to_string(),
    }
}
