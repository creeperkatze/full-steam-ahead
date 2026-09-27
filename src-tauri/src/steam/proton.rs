use crate::{
    error::{io_context, AppResult},
    models::BackupPlan,
};
use keyvalues_parser::{Obj, Value};
use std::{borrow::Cow, collections::BTreeMap, fs, path::Path};

const DEFAULT_COMPAT_TOOL: &str = "proton_experimental";
const MAPPING_KEY: &str = "CompatToolMapping";

/// Forces Steam to run each of `app_ids` through Proton, by adding a
/// `CompatToolMapping` entry to `config/config.vdf`.
///
/// `backup_dir` is optional. When set, the original `config.vdf` is copied
/// there before the write. The returned plan records that copy, which is what
/// a restore replays.
pub fn setup_compat_tool_mapping(
    install_path: &Path,
    app_ids: &[u32],
    backup_dir: Option<&Path>,
) -> AppResult<Option<BackupPlan>> {
    if app_ids.is_empty() {
        return Ok(None);
    }

    let config_path = install_path.join("config").join("config.vdf");
    let Ok(content) = fs::read_to_string(&config_path) else {
        tracing::warn!(path = %config_path.display(), "config.vdf not found; skipping Proton setup");
        return Ok(None);
    };

    let Some(updated) = add_missing_compat_tool_entries(&content, app_ids) else {
        return Ok(None);
    };

    let backup = match backup_dir {
        Some(backup_dir) => {
            let backup_path = backup_dir.join("config.vdf");
            match fs::copy(&config_path, &backup_path) {
                Ok(_) => Some(BackupPlan {
                    source: config_path.clone(),
                    destination: backup_path,
                }),
                Err(error) => {
                    tracing::warn!(%error, path = %config_path.display(), "Could not back up config.vdf before Proton setup");
                    None
                }
            }
        }
        None => None,
    };

    fs::write(&config_path, updated).map_err(io_context(&config_path))?;
    Ok(backup)
}

/// The updated file, or `None` when nothing needs to change or the file can't be changed safely.
fn add_missing_compat_tool_entries(vdf: &str, app_ids: &[u32]) -> Option<String> {
    let mut config = keyvalues_parser::parse(vdf)
        .inspect_err(|error| tracing::warn!(%error, "Could not parse config.vdf"))
        .ok()?
        .into_vdf();
    let root = config.value.get_mut_obj()?;

    // More than one section means the file is unusual. Don't guess which one Steam reads.
    if count_mappings(root) != 1 {
        tracing::warn!(
            "Could not find a unique CompatToolMapping section in config.vdf, skipping Proton setup. \
             Force a Steam Play compatibility tool on at least one game manually, then retry."
        );
        return None;
    }
    let mapping = find_mapping(root)?;

    let mut changed = false;
    for id in app_ids {
        let id = id.to_string();
        if !mapping.contains_key(id.as_str()) {
            mapping.insert(Cow::Owned(id), vec![Value::Obj(compat_tool_entry())]);
            changed = true;
        }
    }
    changed.then(|| config.to_string())
}

fn count_mappings(obj: &Obj) -> usize {
    obj.iter()
        .flat_map(|(key, values)| {
            values
                .iter()
                .filter_map(Value::get_obj)
                .map(move |child| usize::from(*key == MAPPING_KEY) + count_mappings(child))
        })
        .sum()
}

fn find_mapping<'a, 'text>(obj: &'a mut Obj<'text>) -> Option<&'a mut Obj<'text>> {
    for (key, values) in obj.iter_mut() {
        for value in values {
            let Some(child) = value.get_mut_obj() else {
                continue;
            };
            if *key == MAPPING_KEY {
                return Some(child);
            }
            if let Some(found) = find_mapping(child) {
                return Some(found);
            }
        }
    }
    None
}

fn compat_tool_entry() -> Obj<'static> {
    let field = |value: &'static str| vec![Value::Str(Cow::Borrowed(value))];
    Obj(BTreeMap::from([
        (Cow::Borrowed("name"), field(DEFAULT_COMPAT_TOOL)),
        (Cow::Borrowed("config"), field("")),
        (Cow::Borrowed("Priority"), field("250")),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(entries: &str) -> String {
        format!(
            "\"InstallConfigStore\"\n\
             {{\n\
             \t\"Software\"\n\
             \t{{\n\
             \t\t\"Language\"\t\t\"english\"\n\
             \t\t\"CompatToolMapping\"\n\
             \t\t{{\n\
             {entries}\
             \t\t}}\n\
             \t}}\n\
             }}\n"
        )
    }

    fn entry(id: &str, name: &str) -> String {
        format!(
            "\t\t\t\"{id}\"\n\
             \t\t\t{{\n\
             \t\t\t\t\"name\"\t\t\"{name}\"\n\
             \t\t\t\t\"config\"\t\t\"\"\n\
             \t\t\t\t\"Priority\"\t\t\"250\"\n\
             \t\t\t}}\n"
        )
    }

    // The compat tool of every entry in the mapping, by app id.
    fn mapping(vdf: &str) -> Vec<(String, String)> {
        let mut config = keyvalues_parser::parse(vdf).unwrap().into_vdf();
        let mapping = find_mapping(config.value.get_mut_obj().unwrap()).unwrap();
        mapping
            .iter()
            .map(|(id, values)| {
                let tool = values[0].get_obj().unwrap()["name"][0].get_str().unwrap();
                (id.to_string(), tool.to_string())
            })
            .collect()
    }

    #[test]
    fn adds_entry_to_empty_section() {
        let updated = add_missing_compat_tool_entries(&sample_config(""), &[42]).unwrap();
        assert_eq!(
            mapping(&updated),
            [("42".to_string(), DEFAULT_COMPAT_TOOL.to_string())]
        );
    }

    #[test]
    fn preserves_existing_entries_and_settings_when_adding() {
        let vdf = sample_config(&entry("1", "proton_9"));
        let updated = add_missing_compat_tool_entries(&vdf, &[2]).unwrap();
        assert_eq!(
            mapping(&updated),
            [
                ("1".to_string(), "proton_9".to_string()),
                ("2".to_string(), DEFAULT_COMPAT_TOOL.to_string())
            ]
        );
        assert!(updated.contains("\"Language\"\t\"english\""));
    }

    #[test]
    fn nothing_to_add_is_no_change() {
        let vdf = sample_config(&entry("42", "proton_9"));
        assert!(add_missing_compat_tool_entries(&vdf, &[42]).is_none());
    }

    #[test]
    fn adds_multiple_missing_ids() {
        let updated = add_missing_compat_tool_entries(&sample_config(""), &[1, 2]).unwrap();
        assert_eq!(mapping(&updated).len(), 2);
    }

    #[test]
    fn returns_none_when_section_absent() {
        let vdf = "\"InstallConfigStore\"\n{\n\t\"Software\"\n\t{\n\t}\n}\n";
        assert!(add_missing_compat_tool_entries(vdf, &[42]).is_none());
    }

    #[test]
    fn rejects_a_duplicate_compat_tool_mapping_section() {
        let vdf = sample_config(&entry("1", "proton_9")).replace(
            "\t\"Software\"\n",
            "\t\"Other\"\n\t{\n\t\t\"CompatToolMapping\"\n\t\t{\n\t\t}\n\t}\n\t\"Software\"\n",
        );
        assert!(add_missing_compat_tool_entries(&vdf, &[2]).is_none());
    }

    #[test]
    fn empty_app_ids_is_a_noop() {
        assert!(
            setup_compat_tool_mapping(Path::new("/nonexistent"), &[], None)
                .unwrap()
                .is_none()
        );
    }
}
