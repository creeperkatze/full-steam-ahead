use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
    process,
};
use serde::Deserialize;
use std::{path::Path, process::Command};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let output = process::command_output_no_window(Command::new("powershell").args([
        "/NoProfile",
        "/Command",
        GAME_PASS_SCRIPT,
    ]))
    .map_err(|source| AppError::Io {
        path: "powershell".into(),
        source,
    })?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let apps = match serde_json::from_str::<Vec<AppxInfo>>(&raw) {
        Ok(apps) => apps,
        Err(_) => serde_json::from_str::<AppxInfo>(&raw)
            .map(|app| vec![app])
            .unwrap_or_default(),
    };
    let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let explorer = Path::new(&windows_dir).join("explorer.exe");

    Ok(apps
        .into_iter()
        .filter(|app| app.kind.is_game())
        .filter(|app| {
            !app.display_name.contains("DisplayName")
                && !app.display_name.contains("ms-resource")
                && !app.display_name.trim().is_empty()
        })
        .map(|app| {
            launcher_candidate(
                user,
                ImportSource::GamePass,
                "gamepass",
                app.display_name,
                explorer.clone(),
                format!(
                    "shell:AppsFolder\\{}!{}",
                    app.family_name,
                    app.kind.as_ref()
                ),
                vec!["Game Pass".to_string()],
            )
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct AppxInfo {
    kind: AppxKind,
    display_name: String,
    family_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AppxKind {
    Null,
    Str(String),
    Array(Vec<String>),
}

impl AppxKind {
    fn as_ref(&self) -> &str {
        match self {
            AppxKind::Null => "",
            AppxKind::Str(value) => value,
            AppxKind::Array(values) => values.first().map(String::as_str).unwrap_or(""),
        }
    }

    fn is_game(&self) -> bool {
        match self {
            AppxKind::Str(value) => value == "Game",
            AppxKind::Array(values) => values.iter().any(|value| value == "Game"),
            _ => false,
        }
    }
}

const GAME_PASS_SCRIPT: &str = r#"
Get-AppxPackage |
Where-Object { -not $_.IsFramework } |
ForEach-Object {
    try {
        $manifest = Get-AppxPackageManifest $_
        $application = $manifest.Package.Applications.Application
        [PSCustomObject]@{
            kind = $application.Id
            display_name = $manifest.Package.Properties.DisplayName
            family_name = $_.PackageFamilyName
        }
    } catch {}
} |
ConvertTo-Json -Depth 5
"#;

#[cfg(test)]
mod tests {
    use super::*;

    // AppxKind::is_game

    #[test]
    fn is_game_true_for_str_game() {
        assert!(AppxKind::Str("Game".to_string()).is_game());
    }

    #[test]
    fn is_game_false_for_non_game_str() {
        assert!(!AppxKind::Str("Framework".to_string()).is_game());
    }

    #[test]
    fn is_game_true_when_array_contains_game() {
        assert!(AppxKind::Array(vec!["DeveloperTools".to_string(), "Game".to_string()]).is_game());
    }

    #[test]
    fn is_game_false_for_empty_array() {
        assert!(!AppxKind::Array(vec![]).is_game());
    }

    #[test]
    fn is_game_false_for_null() {
        assert!(!AppxKind::Null.is_game());
    }

    // AppxKind::as_ref

    #[test]
    fn as_ref_returns_str_value() {
        assert_eq!(AppxKind::Str("App".to_string()).as_ref(), "App");
    }

    #[test]
    fn as_ref_returns_first_array_element() {
        assert_eq!(
            AppxKind::Array(vec!["First".to_string(), "Second".to_string()]).as_ref(),
            "First"
        );
    }

    #[test]
    fn as_ref_returns_empty_for_null() {
        assert_eq!(AppxKind::Null.as_ref(), "");
    }

    #[test]
    fn as_ref_returns_empty_for_empty_array() {
        assert_eq!(AppxKind::Array(vec![]).as_ref(), "");
    }

    // JSON deserialization

    #[test]
    fn deserializes_game_with_str_kind() {
        let json = r#"{"kind":"Game","display_name":"My Game","family_name":"Pub.MyGame_abc"}"#;
        let info: AppxInfo = serde_json::from_str(json).unwrap();
        assert!(info.kind.is_game());
        assert_eq!(info.display_name, "My Game");
    }

    #[test]
    fn deserializes_game_with_array_kind() {
        let json = r#"{"kind":["Framework","Game"],"display_name":"App","family_name":"A_bc"}"#;
        let info: AppxInfo = serde_json::from_str(json).unwrap();
        assert!(info.kind.is_game());
    }

    #[test]
    fn deserializes_non_game_with_null_kind() {
        let json = r#"{"kind":null,"display_name":"Tool","family_name":"T_bc"}"#;
        let info: AppxInfo = serde_json::from_str(json).unwrap();
        assert!(!info.kind.is_game());
    }

    #[test]
    fn deserializes_array_of_infos() {
        let json = r#"[
            {"kind":"Game","display_name":"A","family_name":"A_1"},
            {"kind":"Framework","display_name":"B","family_name":"B_2"}
        ]"#;
        let apps: Vec<AppxInfo> = serde_json::from_str(json).unwrap();
        assert_eq!(apps.iter().filter(|a| a.kind.is_game()).count(), 1);
    }
}
