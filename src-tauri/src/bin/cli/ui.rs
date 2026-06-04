use console::style;
use full_steam_ahead_lib::models::{ChangeKind, PreviewPlan};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// Generated from assets/logo.png at compile time by build.rs.
const LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.txt"));

pub fn print_header() {
    println!("\n{LOGO}\n");
    println!(
        "  {}",
        style("Import games from other launchers into Steam").dim()
    );
    println!();
}

pub fn make_spinner(msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.into());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn print_summary(plan: &PreviewPlan) {
    let new_c = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::AddShortcut))
        .count();
    let upd_c = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::UpdateShortcut))
        .count();
    let art_c = plan
        .changes
        .iter()
        .filter(|c| matches!(c.kind, ChangeKind::WriteArtwork))
        .count();

    if new_c > 0 {
        println!("  {} {} new shortcut(s)", style("+").green().bold(), new_c);
    }
    if upd_c > 0 {
        println!("  {} {} updated shortcut(s)", style("~").yellow().bold(), upd_c);
    }
    if art_c > 0 {
        println!("  {} {} artwork file(s)", style("◆").cyan(), art_c);
    }
    if !plan.backups.is_empty() {
        println!(
            "  {} {} backup(s) will be created",
            style("·").dim(),
            plan.backups.len()
        );
    }
    println!();
}
