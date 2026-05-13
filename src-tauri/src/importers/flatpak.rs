use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::process::Command;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let output = Command::new("flatpak")
        .args(["list", "--app", "--columns=name,application"])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    let text = String::from_utf8_lossy(&output.stdout);
    let candidates = text
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let name = parts.next()?.trim().to_string();
            let app_id = parts.next()?.trim().to_string();
            if name.is_empty() || app_id.is_empty() {
                return None;
            }
            Some(launcher_candidate(
                user,
                ImportSource::Flatpak,
                "flatpak",
                name,
                "flatpak".into(),
                format!("run {app_id}"),
                vec!["Flatpak".to_string()],
            ))
        })
        .collect();

    Ok(candidates)
}
