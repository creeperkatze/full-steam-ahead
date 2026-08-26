use crate::error::{io_context, AppResult};
use std::{fs, path::Path};

/// Compatibility tool id available on every Linux Steam client.
const DEFAULT_COMPAT_TOOL: &str = "proton_experimental";

/// Ensures each of `app_ids` has a `CompatToolMapping` entry in `config/config.vdf`, forcing Steam to run that shortcut through Proton.
pub fn setup_compat_tool_mapping(install_path: &Path, app_ids: &[u32]) -> AppResult<()> {
    if app_ids.is_empty() {
        return Ok(());
    }

    let config_path = install_path.join("config").join("config.vdf");
    let Ok(content) = fs::read_to_string(&config_path) else {
        tracing::warn!(path = %config_path.display(), "config.vdf not found; skipping Proton setup");
        return Ok(());
    };

    let ids: Vec<String> = app_ids.iter().map(u32::to_string).collect();
    let Some(updated) = add_missing_compat_tool_entries(&content, &ids) else {
        tracing::warn!(
            "Could not find a CompatToolMapping section in config.vdf; skipping Proton setup. \
             Force a Steam Play compatibility tool on at least one game manually, then retry."
        );
        return Ok(());
    };

    if updated == content {
        return Ok(());
    }

    fs::write(&config_path, updated).map_err(io_context(&config_path))
}

struct CompatToolMappingSection {
    /// Byte offset where the section's entries begin.
    entries_start: usize,
    /// Byte offset right after the last entry's content, where a new entry can be inserted.
    entries_end: usize,
    /// Indentation (in tabs) of each entry's `"<appid>"` key line.
    entry_indent: usize,
}

fn find_compat_tool_mapping_section(vdf: &str) -> Option<CompatToolMappingSection> {
    const KEY: &str = "\"CompatToolMapping\"\n";

    let key_start = vdf.find(KEY)?;
    let after_key = &vdf[key_start + KEY.len()..];

    // The tab count before the opening brace is the section's indentation.
    let open_brace_offset = after_key.find('{')?;
    let section_indent = open_brace_offset;
    let entry_indent = section_indent + 1;

    let entries_start = key_start + KEY.len() + open_brace_offset + "{\n".len();
    let rest = &vdf[entries_start..];

    // The section's closing brace shares its opening brace's indentation, so this skips nested entry blocks (which close one level deeper).
    let close_line = format!("{}}}", "\t".repeat(section_indent));
    let entries_end = if rest.starts_with(&close_line) {
        entries_start
    } else {
        let marker = format!("\n{close_line}");
        entries_start + rest.find(&marker)? + 1
    };

    Some(CompatToolMappingSection {
        entries_start,
        entries_end,
        entry_indent,
    })
}

fn add_missing_compat_tool_entries(vdf: &str, app_ids: &[String]) -> Option<String> {
    let section = find_compat_tool_mapping_section(vdf)?;
    let entries = &vdf[section.entries_start..section.entries_end];

    let missing: Vec<&String> = app_ids
        .iter()
        .filter(|id| !entries.contains(&format!("\"{id}\"\n")))
        .collect();
    if missing.is_empty() {
        return Some(vdf.to_string());
    }

    let indent = "\t".repeat(section.entry_indent);
    let field_indent = "\t".repeat(section.entry_indent + 1);
    let mut new_entries = String::new();
    for id in missing {
        new_entries.push_str(&format!(
            "{indent}\"{id}\"\n{indent}{{\n\
             {field_indent}\"name\"\t\t\"{DEFAULT_COMPAT_TOOL}\"\n\
             {field_indent}\"config\"\t\t\"\"\n\
             {field_indent}\"Priority\"\t\t\"250\"\n\
             {indent}}}\n"
        ));
    }

    Some(format!(
        "{}{}{}",
        &vdf[..section.entries_end],
        new_entries,
        &vdf[section.entries_end..]
    ))
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

    #[test]
    fn finds_section_bounds_with_no_existing_entries() {
        let vdf = sample_config("");
        let section = find_compat_tool_mapping_section(&vdf).unwrap();
        assert_eq!(section.entry_indent, 3);
        assert_eq!(section.entries_start, section.entries_end);
        assert_eq!(&vdf[section.entries_start..section.entries_end], "");
    }

    #[test]
    fn finds_section_bounds_with_existing_entries() {
        let vdf = sample_config(&entry("42", "proton_9"));
        let section = find_compat_tool_mapping_section(&vdf).unwrap();
        assert_eq!(
            &vdf[section.entries_start..section.entries_end],
            entry("42", "proton_9")
        );
    }

    #[test]
    fn returns_none_when_section_absent() {
        let vdf = "\"InstallConfigStore\"\n{\n\t\"Software\"\n\t{\n\t}\n}\n";
        assert!(find_compat_tool_mapping_section(vdf).is_none());
    }

    #[test]
    fn adds_entry_to_empty_section() {
        let vdf = sample_config("");
        let updated = add_missing_compat_tool_entries(&vdf, &["42".to_string()]).unwrap();
        assert!(updated.contains(&format!(
            "\"42\"\n\t\t\t{{\n\t\t\t\t\"name\"\t\t\"{DEFAULT_COMPAT_TOOL}\""
        )));
        let reparsed = find_compat_tool_mapping_section(&updated).unwrap();
        assert!(!updated[reparsed.entries_start..reparsed.entries_end].is_empty());
    }

    #[test]
    fn preserves_existing_entries_when_adding() {
        let vdf = sample_config(&entry("1", "proton_experimental"));
        let updated = add_missing_compat_tool_entries(&vdf, &["2".to_string()]).unwrap();
        assert!(updated.contains("\"1\""));
        assert!(updated.contains("\"2\""));
    }

    #[test]
    fn skips_ids_that_already_have_an_entry() {
        let vdf = sample_config(&entry("42", "proton_experimental"));
        let updated = add_missing_compat_tool_entries(&vdf, &["42".to_string()]).unwrap();
        assert_eq!(updated, vdf, "must not duplicate an existing entry");
    }

    #[test]
    fn adds_multiple_missing_ids() {
        let vdf = sample_config("");
        let updated =
            add_missing_compat_tool_entries(&vdf, &["1".to_string(), "2".to_string()]).unwrap();
        assert!(updated.contains("\"1\""));
        assert!(updated.contains("\"2\""));
    }

    #[test]
    fn returns_none_when_no_section_and_ids_present() {
        let vdf = "\"InstallConfigStore\"\n{\n}\n";
        assert!(add_missing_compat_tool_entries(vdf, &["42".to_string()]).is_none());
    }

    #[test]
    fn empty_app_ids_is_a_noop() {
        setup_compat_tool_mapping(Path::new("/nonexistent"), &[]).unwrap();
    }

    #[test]
    fn result_reparses_cleanly_after_multiple_additions() {
        let vdf = sample_config(&entry("1", "proton_experimental"));
        let updated =
            add_missing_compat_tool_entries(&vdf, &["2".to_string(), "3".to_string()]).unwrap();
        let section = find_compat_tool_mapping_section(&updated).unwrap();
        let body = &updated[section.entries_start..section.entries_end];
        assert!(body.contains("\"1\""));
        assert!(body.contains("\"2\""));
        assert!(body.contains("\"3\""));
        // Two tabs is the section's own closing brace, not a nested entry's (three tabs).
        assert!(updated[section.entries_end..].starts_with("\t\t}"));
    }
}
