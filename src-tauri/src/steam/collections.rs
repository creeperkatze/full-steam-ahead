use crate::{
    error::{io_context, AppError, AppResult},
    models::ImportCandidate,
    steam::candidate_app_id,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const APP_COLLECTION_PREFIX: &str = "fsa";

pub fn update_modern_collections(path: &Path, candidates: &[ImportCandidate]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }

    let (mut collections, preserve_control_prefix) = if path.exists() {
        let raw = fs::read_to_string(path).map_err(io_context(path))?;
        parse_cloud_collections(&raw, path)?
    } else {
        (Vec::new(), false)
    };

    let mut grouped: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for candidate in candidates {
        grouped
            .entry(candidate.source.collection_name())
            .or_default()
            .push(candidate_app_id(candidate));
    }
    add_to_managed_collections(&mut collections, grouped);

    let mut serialized = serde_json::to_string(&collections).map_err(|source| AppError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    if preserve_control_prefix {
        serialized.insert(0, '\u{1}');
    }

    fs::write(path, serialized).map_err(io_context(path))
}

fn parse_cloud_collections(raw: &str, path: &Path) -> AppResult<(Vec<(String, Value)>, bool)> {
    let preserve_control_prefix = raw.starts_with('\u{1}');
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok((Vec::new(), preserve_control_prefix));
    }

    let without_prefix = raw.strip_prefix('\u{1}').unwrap_or(raw);
    let value = serde_json::from_str::<Value>(without_prefix).map_err(|source| AppError::Json {
        path: path.to_path_buf(),
        source,
    })?;

    match value {
        Value::Array(_) => {
            let parsed =
                serde_json::from_value::<Vec<(String, Value)>>(value).map_err(|source| {
                    AppError::Json {
                        path: path.to_path_buf(),
                        source,
                    }
                })?;
            Ok((parsed, preserve_control_prefix))
        }
        Value::Null => Ok((Vec::new(), preserve_control_prefix)),
        _ => Err(AppError::Message(
            "Steam cloud collections file has an unsupported JSON shape; not overwriting it."
                .to_string(),
        )),
    }
}

// Games imported before stay, and collections of other sources are left alone.
fn add_to_managed_collections(
    collections: &mut Vec<(String, Value)>,
    grouped: BTreeMap<String, Vec<u32>>,
) {
    for (source, app_ids) in grouped {
        let key = format!("user-collections.{}", managed_collection_id(&source));
        let position = collections.iter().position(|(k, _)| *k == key);
        let mut added = position
            .and_then(|i| read_collection(&key, &collections[i].1))
            .map(|collection| collection.added)
            .unwrap_or_default();
        for id in app_ids {
            if !added.contains(&id) {
                added.push(id);
            }
        }

        let entry = managed_collection_entry(&source, &added);
        match position {
            Some(i) => collections[i] = entry,
            None => collections.push(entry),
        }
    }
}

fn read_collection(key: &str, entry: &Value) -> Option<SteamCollectionValue> {
    let raw = entry.get("value")?.as_str()?;
    serde_json::from_str(raw)
        .inspect_err(|error| tracing::warn!(key, %error, "Skipping unreadable Steam collection"))
        .ok()
}

fn managed_collection_entry(source: &str, app_ids: &[u32]) -> (String, Value) {
    let id = managed_collection_id(source);
    let key = format!("user-collections.{id}");
    let value = SteamCollectionValue {
        id,
        name: source.to_string(),
        added: app_ids.to_vec(),
        removed: Vec::new(),
    };

    (
        key.clone(),
        json!({
            "key": key,
            "timestamp": current_timestamp(),
            "value": serde_json::to_string(&value).unwrap_or_default(),
            "conflictResolutionMethod": "custom",
            "strMethodId": "union-collections"
        }),
    )
}

fn is_managed_key(key: &str) -> bool {
    key.starts_with(&format!("user-collections.{APP_COLLECTION_PREFIX}-"))
}

fn managed_collection_id(source: &str) -> String {
    let slug = source
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character.is_whitespace() || matches!(character, '-' | '_' | ':') {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    format!(
        "{APP_COLLECTION_PREFIX}-{}",
        if slug.is_empty() { "imported" } else { &slug }
    )
}

pub fn existing_managed_app_ids(path: &Path) -> HashMap<String, HashSet<u32>> {
    if !path.exists() {
        return HashMap::new();
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "Could not read Steam collections");
            return HashMap::new();
        }
    };
    let entries = match parse_cloud_collections(&raw, path) {
        Ok((entries, _)) => entries,
        Err(error) => {
            tracing::warn!(%error, "Could not parse Steam collections");
            return HashMap::new();
        }
    };
    let mut result = HashMap::new();
    for (key, value) in entries {
        if !is_managed_key(&key) {
            continue;
        }
        if let Some(coll) = read_collection(&key, &value) {
            result.insert(coll.name, coll.added.into_iter().collect());
        }
    }
    result
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Serialize, Deserialize)]
struct SteamCollectionValue {
    id: String,
    name: String,
    added: Vec<u32>,
    removed: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn added_ids(collections: &[(String, Value)], source: &str) -> Vec<u32> {
        let key = format!("user-collections.{}", managed_collection_id(source));
        let (_, entry) = collections.iter().find(|(k, _)| *k == key).unwrap();
        read_collection(&key, entry).unwrap().added
    }

    #[test]
    fn importing_one_source_keeps_the_other_collections() {
        let mut collections = Vec::new();
        add_to_managed_collections(
            &mut collections,
            BTreeMap::from([("Epic".into(), vec![1, 2])]),
        );
        add_to_managed_collections(&mut collections, BTreeMap::from([("GOG".into(), vec![3])]));
        assert_eq!(added_ids(&collections, "Epic"), [1, 2]);
        assert_eq!(added_ids(&collections, "GOG"), [3]);
    }

    #[test]
    fn importing_again_keeps_games_imported_before() {
        let mut collections = vec![("user-collections.favorite".to_string(), json!({}))];
        add_to_managed_collections(
            &mut collections,
            BTreeMap::from([("Epic".into(), vec![1, 2])]),
        );
        add_to_managed_collections(
            &mut collections,
            BTreeMap::from([("Epic".into(), vec![2, 3])]),
        );
        assert_eq!(collections.len(), 2);
        assert_eq!(added_ids(&collections, "Epic"), [1, 2, 3]);
    }

    // managed_collection_id

    #[test]
    fn collection_id_simple_name() {
        assert_eq!(managed_collection_id("Epic Games"), "fsa-epic-games");
    }

    #[test]
    fn collection_id_collapses_separators() {
        assert_eq!(
            managed_collection_id("Ubisoft  Connect"),
            "fsa-ubisoft-connect"
        );
    }

    #[test]
    fn collection_id_empty_falls_back_to_imported() {
        assert_eq!(managed_collection_id(""), "fsa-imported");
    }

    #[test]
    fn collection_id_only_special_chars_falls_back_to_imported() {
        assert_eq!(managed_collection_id("!@#$%"), "fsa-imported");
    }

    #[test]
    fn collection_id_preserves_alphanumerics() {
        assert_eq!(managed_collection_id("GOG"), "fsa-gog");
    }

    // is_managed_key

    #[test]
    fn managed_key_matches_fsa_prefix() {
        assert!(is_managed_key("user-collections.fsa-epic-games"));
    }

    #[test]
    fn managed_key_rejects_unrelated_key() {
        assert!(!is_managed_key("user-collections.favorites"));
    }

    #[test]
    fn managed_key_rejects_partial_prefix() {
        assert!(!is_managed_key("user-collections.fsa"));
    }

    // parse_cloud_collections

    #[test]
    fn parse_empty_string_returns_empty() {
        let (entries, prefix) = parse_cloud_collections("", Path::new("x.json")).unwrap();
        assert!(entries.is_empty());
        assert!(!prefix);
    }

    #[test]
    fn parse_null_json_returns_empty() {
        let (entries, _) = parse_cloud_collections("null", Path::new("x.json")).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_array_returns_entries() {
        let json = r#"[["some-key", {"value": "data"}]]"#;
        let (entries, _) = parse_cloud_collections(json, Path::new("x.json")).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "some-key");
    }

    #[test]
    fn parse_strips_control_prefix_and_flags_it() {
        let json = "\x01[[\"k\", {}]]";
        let (entries, preserve) = parse_cloud_collections(json, Path::new("x.json")).unwrap();
        assert!(preserve);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn parse_non_array_non_null_returns_error() {
        let result = parse_cloud_collections("{\"key\": 1}", Path::new("x.json"));
        assert!(result.is_err());
    }
}
