use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{path::Path, process::Command};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let output = Command::new("powershell")
        .args(["/NoProfile", "/Command", GAME_PASS_SCRIPT])
        .output()
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
                format!("shell:AppsFolder\\{}!{}", app.family_name, app.kind.as_ref()),
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
