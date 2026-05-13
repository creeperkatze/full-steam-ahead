use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{collections::HashMap, process::Command};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let output = Command::new("flatpak")
        .args([
            "run",
            "--command=bottles-cli",
            "com.usebottles.bottles",
            "-j",
            "list",
            "bottles",
        ])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    let text = String::from_utf8_lossy(&output.stdout);
    let Ok(bottles_map) = serde_json::from_str::<HashMap<String, Bottle>>(&text) else {
        return Ok(Vec::new());
    };

    let candidates = bottles_map
        .into_values()
        .flat_map(|bottle| {
            let bottle_name = bottle.name.clone();
            bottle.external_programs.into_values().map(move |program| {
                launcher_candidate(
                    user,
                    ImportSource::Bottles,
                    "bottles",
                    program.name.clone(),
                    "flatpak".into(),
                    format!(
                        "run --command=bottles-cli com.usebottles.bottles run --args-replace -b \"{}\" -p \"{}\"",
                        bottle_name, program.name
                    ),
                    vec!["Bottles".to_string()],
                )
            })
        })
        .collect();

    Ok(candidates)
}

#[derive(Deserialize)]
struct Bottle {
    #[serde(alias = "Name")]
    name: String,
    #[serde(alias = "External_Programs")]
    external_programs: HashMap<String, Program>,
}

#[derive(Deserialize)]
struct Program {
    #[serde(alias = "Name")]
    name: String,
}
