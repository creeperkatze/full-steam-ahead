mod settings;
mod steps;
mod ui;

use console::style;
use dialoguer::theme::ColorfulTheme;
use full_steam_ahead_lib::error::AppResult;

fn main() {
    ui::print_header();
    if let Err(e) = run() {
        eprintln!("\n{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let theme = ColorfulTheme::default();

    let user = steps::detect_and_select_user(&theme)?;

    let candidates = steps::scan_sources(&user)?;
    if candidates.is_empty() {
        println!("{} No games found.", style("—").dim());
        return Ok(());
    }

    let selected = steps::select_games(&theme, candidates)?;
    if selected.is_empty() {
        println!("{} No games selected.", style("—").dim());
        return Ok(());
    }

    let (plan, options) = steps::build_plan(&user, &selected)?;
    ui::print_summary(&plan);

    if plan.changes.is_empty() {
        println!("{} Everything is already up to date.", style("✓").green());
        return Ok(());
    }

    if !steps::confirm_apply(&theme)? {
        println!("{} Aborted.", style("—").dim());
        return Ok(());
    }

    println!();
    let result = steps::apply_changes(plan, selected, options)?;

    println!(
        "\n  {}   {} change(s) applied,  {} backup(s) created",
        style("Done!").green().bold(),
        style(result.applied_changes.len()).bold(),
        result.backups_created.len()
    );

    Ok(())
}
