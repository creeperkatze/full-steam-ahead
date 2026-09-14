use crate::{
    error::{AppError, AppResult},
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
    process,
};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn scan(user: &SteamUser, _custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
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
    let apps = serde_json::from_str::<Vec<AppxInfo>>(&raw).unwrap_or_default();
    let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let explorer = Path::new(&windows_dir).join("explorer.exe");

    Ok(apps
        .into_iter()
        .filter_map(|app| {
            let uri = app.launch_uri()?;
            let icon = super::icons::package_icon(&app.install_location, &app.icon);
            let mut candidate = launcher_candidate(
                user,
                ImportSource::GamePass,
                "gamepass",
                app.display_name,
                explorer.clone(),
                uri,
                vec!["Game Pass".to_string()],
            );
            if let Some(icon) = icon {
                crate::steam::artwork::prefer_local_icon(&mut candidate.artwork, &icon);
            }
            Some(candidate)
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct AppxInfo {
    app_id: String,
    is_game: bool,
    display_name: String,
    family_name: String,
    #[serde(default)]
    install_location: PathBuf,
    #[serde(default)]
    icon: String,
}

impl AppxInfo {
    fn launch_uri(&self) -> Option<String> {
        if !self.is_game
            || self.app_id.trim().is_empty()
            || self.family_name.trim().is_empty()
            || self.display_name.contains("DisplayName")
            || self.display_name.contains("ms-resource")
            || self.display_name.trim().is_empty()
        {
            return None;
        }
        Some(format!(
            "shell:AppsFolder\\{}!{}",
            self.family_name, self.app_id
        ))
    }
}

const GAME_PASS_SCRIPT: &str = r#"
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$apps = @(Get-AppxPackage |
Where-Object { -not $_.IsFramework -and -not $_.IsResourcePackage } |
ForEach-Object {
    try {
        $package = $_
        $manifest = Get-AppxPackageManifest $package -ErrorAction Stop
        $gameIds = @()
        # GDK application IDs are developer-defined, not necessarily "Game".
        try {
            $configPath = Join-Path $package.InstallLocation 'MicrosoftGame.config'
            if (Test-Path -LiteralPath $configPath) {
                [xml]$config = Get-Content -LiteralPath $configPath -Raw -ErrorAction Stop
                $gameIds = @($config.Game.ExecutableList.Executable |
                    Where-Object { $_.IsDevOnly -ne 'true' -and $_.Id } |
                    ForEach-Object { $_.Id })
            }
        } catch {}
        foreach ($application in $manifest.Package.Applications.Application) {
            if (-not $application.Id -or $application.VisualElements.AppListEntry -eq 'none') {
                continue
            }
            $displayName = $application.VisualElements.DisplayName
            if ([string]::IsNullOrWhiteSpace($displayName) -or $displayName -like 'ms-resource:*') {
                $displayName = $manifest.Package.Properties.DisplayName
            }
            $icon = $application.VisualElements.Square44x44Logo
            if (-not $icon) { $icon = $application.VisualElements.Square150x150Logo }
            [PSCustomObject]@{
                app_id = [string]$application.Id
                is_game = $application.Id -ceq 'Game' -or $gameIds -ccontains $application.Id
                display_name = [string]$displayName
                family_name = $package.PackageFamilyName
                install_location = $package.InstallLocation
                icon = [string]$icon
            }
        }
    } catch {}
})
ConvertTo-Json -InputObject $apps -Depth 5
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn app(app_id: &str, is_game: bool, display_name: &str) -> AppxInfo {
        AppxInfo {
            app_id: app_id.to_string(),
            is_game,
            display_name: display_name.to_string(),
            family_name: "Publisher.Game_abc".to_string(),
            install_location: PathBuf::new(),
            icon: String::new(),
        }
    }

    #[test]
    fn manifest_scan_exports_game_logo_not_helper_logo() {
        let script = format!(
            r#"
function Get-AppxPackage {{
    [PSCustomObject]@{{ IsFramework = $false; PackageFamilyName = 'Fixture_abc'; InstallLocation = 'C:\Games\Fixture' }}
}}
function Get-AppxPackageManifest {{
    [xml]'<Package><Properties><DisplayName>Fixture</DisplayName></Properties><Applications><Application Id="Helper"><VisualElements Square44x44Logo="helper.png" /></Application><Application Id="Game"><VisualElements Square44x44Logo="Images\game.png" /></Application></Applications></Package>'
}}
{GAME_PASS_SCRIPT}
"#
        );
        let output = process::command_output_no_window(Command::new("powershell").args([
            "/NoProfile",
            "/Command",
            &script,
        ]))
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let apps: Vec<AppxInfo> = serde_json::from_slice(&output.stdout).unwrap();
        let app = apps.iter().find(|app| app.launch_uri().is_some()).unwrap();
        assert_eq!(app.icon, "Images\\game.png");
        assert_eq!(app.install_location, Path::new("C:\\Games\\Fixture"));
        assert_eq!(app.app_id, "Game");
    }

    #[test]
    fn preserves_legacy_game_uri() {
        assert_eq!(
            app("Game", true, "Legacy game").launch_uri().as_deref(),
            Some("shell:AppsFolder\\Publisher.Game_abc!Game")
        );
    }

    #[test]
    fn preserves_custom_application_id() {
        assert_eq!(
            app("codShip", true, "Modern Warfare 3")
                .launch_uri()
                .as_deref(),
            Some("shell:AppsFolder\\Publisher.Game_abc!codShip")
        );
    }

    #[test]
    fn rejects_non_games_and_incomplete_entries() {
        assert!(app("App", false, "Ordinary app").launch_uri().is_none());
        assert!(app("", true, "Game").launch_uri().is_none());
        let mut missing_family = app("Game", true, "Game");
        missing_family.family_name.clear();
        assert!(missing_family.launch_uri().is_none());
        for name in ["", "  ", "ms-resource:Title", "UnresolvedDisplayName"] {
            assert!(app("Game", true, name).launch_uri().is_none());
        }
    }

    fn scan_fixtures(setup: &str) -> Vec<AppxInfo> {
        let script = format!(
            "{}\n{setup}\n{GAME_PASS_SCRIPT}",
            include_str!("fixtures/gamepass.ps1")
        );
        let output = process::command_output_no_window(Command::new("powershell").args([
            "/NoProfile",
            "/Command",
            &script,
        ]))
        .expect("run PowerShell fixture scan");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("UTF-8 app array")
    }

    #[test]
    fn serializes_empty_scan_as_array() {
        assert!(scan_fixtures("$fixtures = @()").is_empty());
    }

    #[test]
    fn serializes_single_application_as_array() {
        let apps = scan_fixtures("$fixtures = @($fixtures[0])");
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].app_id, "codShip");
        assert!(apps[0].launch_uri().is_some());
    }

    #[test]
    fn exports_each_custom_application_icon_with_size_fallback() {
        let apps = scan_fixtures(
            r#"
$fixtures = @($fixtures[0])
$fixtures[0].Applications = '<Application Id="Helper"><VisualElements Square44x44Logo="helper.png" /></Application><Application Id="codShip"><VisualElements Square44x44Logo="Images\cod.png" Square150x150Logo="wrong.png" /></Application><Application Id="SecondGame"><VisualElements Square150x150Logo="Images\second.png" /></Application>'
$fixtures[0].Config = '<Game><ExecutableList><Executable Id="codShip" /><Executable Id="SecondGame" /></ExecutableList></Game>'
"#,
        );
        let games: Vec<_> = apps
            .iter()
            .filter(|app| app.launch_uri().is_some())
            .collect();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].app_id, "codShip");
        assert_eq!(games[0].icon, "Images\\cod.png");
        assert_eq!(games[1].app_id, "SecondGame");
        assert_eq!(games[1].icon, "Images\\second.png");
        assert!(games
            .iter()
            .all(|app| app.install_location == Path::new("C:\\Fixture\\Gdk")));
    }

    #[test]
    fn scans_gdk_and_legacy_manifest_fixtures() {
        let apps = scan_fixtures("");
        let games: Vec<_> = apps
            .iter()
            .filter(|app| app.launch_uri().is_some())
            .collect();
        let ids: Vec<_> = games.iter().map(|app| app.app_id.as_str()).collect();
        assert_eq!(ids, ["codShip", "Game", "SecondGame", "Game"]);
        assert_eq!(games[0].display_name, "Modern Warfare \u{ae} 3");
        assert_eq!(games[1].display_name, "Legacy title");
        assert_eq!(games[2].display_name, "Second title");
        assert_eq!(games[3].display_name, "Fallback title");
        assert_eq!(
            games[2].launch_uri().as_deref(),
            Some("shell:AppsFolder\\Fixture.Multi_abc!SecondGame")
        );
    }
}
