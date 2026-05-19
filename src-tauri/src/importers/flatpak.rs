use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use std::process::Command;

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let stdout = Command::new("flatpak")
        .args(["list", "--app", "--columns=name,application"])
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();

    let text = String::from_utf8_lossy(&stdout);
    let candidates = text
        .lines()
        .filter_map(|line| {
            let (name, app_id) = parse_flatpak_line(line)?;
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

fn parse_flatpak_line(line: &str) -> Option<(String, String)> {
    let mut parts = line.splitn(2, '\t');
    let name = parts.next()?.trim().to_string();
    let app_id = parts.next()?.trim().to_string();
    if name.is_empty() || app_id.is_empty() {
        return None;
    }
    Some((name, app_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_normal_line() {
        let (name, id) = parse_flatpak_line("Heroic Games Launcher\tcom.heroicgameslauncher.hgl").unwrap();
        assert_eq!(name, "Heroic Games Launcher");
        assert_eq!(id, "com.heroicgameslauncher.hgl");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let (name, id) = parse_flatpak_line("  My App  \t  org.my.App  ").unwrap();
        assert_eq!(name, "My App");
        assert_eq!(id, "org.my.App");
    }

    #[test]
    fn returns_none_when_no_tab() {
        assert!(parse_flatpak_line("AppName org.app.Id").is_none());
    }

    #[test]
    fn returns_none_when_name_empty() {
        assert!(parse_flatpak_line("\torg.app.Id").is_none());
    }

    #[test]
    fn returns_none_when_app_id_empty() {
        assert!(parse_flatpak_line("App Name\t").is_none());
    }

    #[test]
    fn returns_none_for_empty_line() {
        assert!(parse_flatpak_line("").is_none());
    }
}
