//! Fake games for the store screenshots. Scans return these and applying writes nothing.

use crate::{
    importers::{candidate_from_parts, launcher_candidate},
    models::{
        ApplyProgressEvent, ApplyRequest, ApplyResult, ApplyStep, ImportCandidate, ImportSource,
        SteamUser,
    },
};
use std::{path::PathBuf, thread::sleep, time::Duration};

pub const ENABLED: bool = true;

/// A game's name, its executable or launcher, and the launcher's arguments.
type Game = (&'static str, &'static str, Option<&'static str>);

pub fn scan(source: &ImportSource, user: &SteamUser) -> Vec<ImportCandidate> {
    let slug = source.settings_key().unwrap_or("mock");
    games(source)
        .iter()
        .map(|&(name, exe, args)| {
            let tags = vec![source.display_name()];
            let exe = PathBuf::from(exe);
            match args {
                Some(args) => launcher_candidate(
                    user,
                    source.clone(),
                    slug,
                    name.to_string(),
                    exe,
                    args.to_string(),
                    tags,
                ),
                None => {
                    let start_dir = exe.parent().map(PathBuf::from).unwrap_or_default();
                    candidate_from_parts(
                        user,
                        source.clone(),
                        slug,
                        name.to_string(),
                        exe,
                        start_dir,
                        None,
                        tags,
                    )
                }
            }
        })
        .collect()
}

#[cfg(unix)]
fn games(source: &ImportSource) -> &'static [Game] {
    const FLATPAK: &str = "/usr/bin/flatpak";
    match source {
        ImportSource::Heroic => &[
            (
                "Hollow Knight",
                FLATPAK,
                Some("run com.heroicgameslauncher.hgl heroic://launch/gog/1308320804 --no-gui --no-sandbox"),
            ),
            (
                "Outer Wilds",
                FLATPAK,
                Some("run com.heroicgameslauncher.hgl heroic://launch/legendary/Hawthorn --no-gui --no-sandbox"),
            ),
        ],
        ImportSource::Legendary => &[
            ("Satisfactory", "/usr/bin/legendary", Some("launch CrabEA")),
            ("Control", "/usr/bin/legendary", Some("launch Calluna")),
        ],
        ImportSource::Lutris => &[("Terraria", "/usr/bin/lutris", Some("lutris:rungame/terraria"))],
        ImportSource::Itch => &[("Celeste", "/home/user/.config/itch/apps/celeste/Celeste", None)],
        ImportSource::Flatpak => &[("OpenTTD", FLATPAK, Some("run org.openttd.OpenTTD"))],
        ImportSource::MiniGalaxy => &[(
            "Stardew Valley",
            "/home/user/GOG Games/Stardew Valley/start.sh",
            None,
        )],
        _ => &[],
    }
}

#[cfg(not(unix))]
fn games(source: &ImportSource) -> &'static [Game] {
    const EPIC: &str =
        r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe";
    const EXPLORER: &str = r"C:\Windows\explorer.exe";
    match source {
        ImportSource::Epic => &[
            (
                "Hades",
                EPIC,
                Some("com.epicgames.launcher://apps/Min?action=launch&silent=true"),
            ),
            (
                "Control",
                EPIC,
                Some("com.epicgames.launcher://apps/Calluna?action=launch&silent=true"),
            ),
        ],
        ImportSource::Gog => &[
            (
                "The Witcher 3: Wild Hunt",
                r"C:\Program Files (x86)\GOG Galaxy\Games\The Witcher 3 Wild Hunt\bin\x64\witcher3.exe",
                None,
            ),
            (
                "Cyberpunk 2077",
                r"C:\Program Files (x86)\GOG Galaxy\Games\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe",
                None,
            ),
        ],
        ImportSource::Itch => &[(
            "Celeste",
            r"C:\Users\User\AppData\Roaming\itch\apps\celeste\Celeste.exe",
            None,
        )],
        ImportSource::EaApp => &[(
            "It Takes Two",
            r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe",
            Some("origin2://game/launch?offerIds=Origin.OFR.50.0004246"),
        )],
        ImportSource::UbisoftConnect => &[(
            "Rayman Legends",
            r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe",
            Some("uplay://launch/5800/0"),
        )],
        ImportSource::Xbox => &[
            (
                "Forza Horizon 5",
                EXPLORER,
                Some(
                    r"shell:AppsFolder\Microsoft.624F8B84B80_8wekyb3d8bbwe!Microsoft.SunriseBaseGame",
                ),
            ),
            (
                "Hi-Fi RUSH",
                EXPLORER,
                Some(r"shell:AppsFolder\BethesdaSoftworks.HiFiRush_3275kfvn8vcwc!Game"),
            ),
        ],
        ImportSource::Amazon => &[(
            "Fallout: New Vegas",
            r"C:\Program Files\Amazon Games\App\Amazon Games.exe",
            Some("amazon-games://play/amzn1.adg.product.5d1ef1a8"),
        )],
        ImportSource::Playnite => &[(
            "Baldur's Gate 3",
            r"C:\Users\User\AppData\Local\Playnite\Playnite.DesktopApp.exe",
            Some("--start 6f1c2b4e-8a3d-4c55-9b1e-2d7f0a9c3e61"),
        )],
        _ => &[],
    }
}

/// Walks through the apply steps like a real import, without touching Steam.
pub fn apply(on_progress: &impl Fn(ApplyProgressEvent), request: ApplyRequest) -> ApplyResult {
    let mut steps = Vec::new();
    if request.options.stop_steam {
        steps.push(ApplyStep::StoppingSteam);
    }
    steps.push(ApplyStep::CreatingBackups);
    steps.extend(
        request
            .candidates
            .iter()
            .map(|candidate| ApplyStep::ApplyingArtwork {
                game_name: Some(candidate.name.clone()),
            }),
    );
    steps.push(ApplyStep::UpdatingShortcuts);
    steps.push(ApplyStep::UpdatingCollections);
    if request.options.restart_steam {
        steps.push(ApplyStep::RestartingSteam);
    }

    let total = steps.len();
    for (index, step) in steps.into_iter().enumerate() {
        on_progress(ApplyProgressEvent {
            step,
            current: index + 1,
            total,
        });
        sleep(Duration::from_millis(300));
    }

    ApplyResult {
        applied_changes: request.plan.changes,
        backups_created: request
            .plan
            .backups
            .into_iter()
            .map(|backup| backup.destination)
            .collect(),
    }
}
