use crate::{
    error::AppResult,
    importers::{host_binary_path, host_command, launcher_candidate},
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

pub fn scan(user: &SteamUser, custom_path: Option<&Path>) -> AppResult<Vec<ImportCandidate>> {
    // Steam needs an absolute path in the shortcut
    let exe = custom_path
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| host_binary_path("flatpak").display().to_string());

    let stdout = host_command(&exe)
        .args([
            "run",
            "--command=bottles-cli",
            "com.usebottles.bottles",
            "-j",
            "list",
            "bottles",
        ])
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();

    let text = String::from_utf8_lossy(&stdout);
    let Ok(bottles_map) = serde_json::from_str::<HashMap<String, Bottle>>(&text) else {
        return Ok(Vec::new());
    };

    let candidates = bottles_map
        .into_values()
        .flat_map(|bottle| {
            let bottle_name = bottle.name.clone();
            let exe = exe.clone();
            bottle
                .external_programs
                .into_values()
                .filter(|program| !program.removed)
                .map(move |program| {
                    launcher_candidate(
                        user,
                        ImportSource::Bottles,
                        "bottles",
                        program.name.clone(),
                        exe.clone().into(),
                        format!(
                        "run --command=bottles-cli com.usebottles.bottles run -b \"{}\" -p \"{}\"",
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
    /// Set when the user hid the program in Bottles
    #[serde(default)]
    removed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bottle_with_programs() {
        let json = r#"{
            "My Bottle": {
                "Name": "My Bottle",
                "External_Programs": {
                    "prog1": {"Name": "My Game"}
                }
            }
        }"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map["My Bottle"].name, "My Bottle");
        assert_eq!(map["My Bottle"].external_programs["prog1"].name, "My Game");
    }

    #[test]
    fn parses_empty_map() {
        let map: HashMap<String, Bottle> = serde_json::from_str("{}").unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn parses_bottle_with_no_programs() {
        let json = r#"{"B":{"Name":"B","External_Programs":{}}}"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        assert!(map["B"].external_programs.is_empty());
    }

    #[test]
    fn parses_multiple_programs_in_one_bottle() {
        let json = r#"{
            "Bottle": {
                "Name": "Bottle",
                "External_Programs": {
                    "a": {"Name": "Alpha"},
                    "b": {"Name": "Beta"}
                }
            }
        }"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        assert_eq!(map["Bottle"].external_programs.len(), 2);
    }

    #[test]
    fn removed_flag_defaults_to_false() {
        let json = r#"{"B":{"Name":"B","External_Programs":{
            "a": {"name": "Kept"},
            "b": {"name": "Hidden", "removed": true}
        }}}"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        let programs = &map["B"].external_programs;
        assert!(!programs["a"].removed);
        assert!(programs["b"].removed);
    }

    #[test]
    fn lowercase_field_names_also_accepted() {
        // serde alias means both "Name" and "name" work
        let json = r#"{"b":{"name":"b","external_programs":{"p":{"name":"P"}}}}"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        assert_eq!(map["b"].name, "b");
    }
}
