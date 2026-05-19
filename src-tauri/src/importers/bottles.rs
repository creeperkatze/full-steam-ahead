use crate::{
    error::AppResult,
    importers::launcher_candidate,
    models::{ImportCandidate, ImportSource, SteamUser},
};
use serde::Deserialize;
use std::{collections::HashMap, process::Command};

pub fn scan(user: &SteamUser) -> AppResult<Vec<ImportCandidate>> {
    let stdout = Command::new("flatpak")
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
    fn lowercase_field_names_also_accepted() {
        // serde alias means both "Name" and "name" work
        let json = r#"{"b":{"name":"b","external_programs":{"p":{"name":"P"}}}}"#;
        let map: HashMap<String, Bottle> = serde_json::from_str(json).unwrap();
        assert_eq!(map["b"].name, "b");
    }
}
