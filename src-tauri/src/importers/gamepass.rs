use crate::{
    error::AppResult,
    importers::{command_output, launcher_candidate, parse_launcher_json},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{path::Path, process::Command};

pub fn scan(user: &SteamUser, _custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    let Some(output) = command_output(Command::new("powershell").args([
        "/NoProfile",
        "/Command",
        GAME_PASS_SCRIPT,
    ])) else {
        return Ok(Vec::new());
    };
    // The script reports packages it couldn't inspect on stderr and carries on
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        tracing::debug!(message = line.trim(), "Package skipped");
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let Some(apps) = parse_launcher_json::<Vec<AppxInfo>>("Get-AppxPackage script", &raw) else {
        return Ok(Vec::new());
    };
    let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let explorer = Path::new(&windows_dir).join("explorer.exe");

    Ok(apps
        .into_iter()
        .filter_map(|app| {
            let uri = app.launch_uri();
            if app.is_game && uri.is_none() {
                tracing::debug!(?app, "Skipping game with incomplete package details");
            }
            let uri = uri?;
            Some(launcher_candidate(
                user,
                ImportSource::GamePass,
                "gamepass",
                app.display_name,
                explorer.clone(),
                uri,
                vec!["Game Pass".to_string()],
            ))
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct AppxInfo {
    app_id: String,
    is_game: bool,
    display_name: String,
    family_name: String,
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
        } catch {
            [Console]::Error.WriteLine("$($package.Name) MicrosoftGame.config: $_")
        }
        foreach ($application in $manifest.Package.Applications.Application) {
            if (-not $application.Id -or $application.VisualElements.AppListEntry -eq 'none') {
                continue
            }
            $displayName = $application.VisualElements.DisplayName
            if ([string]::IsNullOrWhiteSpace($displayName) -or $displayName -like 'ms-resource:*') {
                $displayName = $manifest.Package.Properties.DisplayName
            }
            [PSCustomObject]@{
                app_id = [string]$application.Id
                is_game = $application.Id -ceq 'Game' -or $gameIds -ccontains $application.Id
                display_name = [string]$displayName
                family_name = $package.PackageFamilyName
            }
        }
    } catch {
        [Console]::Error.WriteLine("$($package.Name): $_")
    }
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
        }
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
        serde_json::from_slice(&run_fixtures(setup).stdout).expect("UTF-8 app array")
    }

    fn run_fixtures(setup: &str) -> std::process::Output {
        let script = format!(
            "{}\n{setup}\n{GAME_PASS_SCRIPT}",
            include_str!("fixtures/gamepass.ps1")
        );
        let output = crate::process::command_output_no_window(Command::new("powershell").args([
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
        output
    }

    #[test]
    fn reports_skipped_packages_on_stderr() {
        let output = run_fixtures("");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("BrokenManifest: Unreadable manifest"),
            "{stderr}"
        );
        assert!(
            stderr.contains("BrokenConfig MicrosoftGame.config:"),
            "{stderr}"
        );
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
