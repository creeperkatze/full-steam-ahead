use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

/// Key paths are relative to `HKEY_LOCAL_MACHINE`; an empty value name reads the default value.
pub trait Registry {
    fn value(&self, key: &str, name: &str) -> Option<String>;
    fn subkeys(&self, key: &str) -> Vec<String>;
    /// Maps a Windows path stored in this registry to a path on this machine.
    fn host_path(&self, windows_path: &str) -> Option<PathBuf>;
}

#[cfg(windows)]
pub struct WindowsRegistry;

#[cfg(windows)]
impl Registry for WindowsRegistry {
    fn value(&self, key: &str, name: &str) -> Option<String> {
        use winreg::{enums::*, RegKey};
        let read = |root, key: &str| {
            RegKey::predef(root)
                .open_subkey(key)
                .ok()?
                .get_value(name)
                .ok()
        };
        const CLASSES: &str = r"SOFTWARE\Classes\";
        read(HKEY_LOCAL_MACHINE, key).or_else(|| {
            // Protocol handlers may be registered per user; HKEY_CLASSES_ROOT merges both
            key.get(..CLASSES.len())
                .filter(|prefix| prefix.eq_ignore_ascii_case(CLASSES))?;
            read(HKEY_CLASSES_ROOT, &key[CLASSES.len()..])
        })
    }

    fn subkeys(&self, key: &str) -> Vec<String> {
        winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            .open_subkey(key)
            .map(|k| k.enum_keys().flatten().collect())
            .unwrap_or_default()
    }

    fn host_path(&self, windows_path: &str) -> Option<PathBuf> {
        Some(PathBuf::from(windows_path))
    }
}

/// The machine hive of a Wine prefix, parsed from its `system.reg`.
pub struct WineRegistry {
    prefix: PathBuf,
    /// Lowercased key path -> (original key path, lowercased value name -> value)
    keys: HashMap<String, (String, HashMap<String, String>)>,
}

impl WineRegistry {
    pub fn parse(prefix: &Path, text: &str) -> Self {
        let mut keys = HashMap::new();
        let mut current: Option<String> = None;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix('[') {
                current = rest.rfind(']').map(|end| {
                    let path = unescape(&rest[..end]);
                    let lower = path.to_lowercase();
                    keys.entry(lower.clone())
                        .or_insert_with(|| (path, HashMap::new()));
                    lower
                });
                continue;
            }
            let Some(key) = &current else {
                continue;
            };
            if let Some((name, value)) = parse_string_value(line) {
                if let Some((_, values)) = keys.get_mut(key) {
                    values.insert(name.to_lowercase(), value);
                }
            }
        }
        Self {
            prefix: prefix.to_path_buf(),
            keys,
        }
    }

    /// Steam's Proton prefixes live in `compatdata/<id>/pfx`; returns `compatdata/<id>`.
    #[cfg_attr(not(unix), allow(dead_code))]
    pub fn proton_compat_folder(&self) -> Option<&Path> {
        (self.prefix.file_name()? == "pfx")
            .then(|| self.prefix.parent())
            .flatten()
    }
}

impl Registry for WineRegistry {
    fn value(&self, key: &str, name: &str) -> Option<String> {
        self.keys
            .get(&key.to_lowercase())?
            .1
            .get(&name.to_lowercase())
            .cloned()
    }

    fn subkeys(&self, key: &str) -> Vec<String> {
        let prefix = format!("{}\\", key.to_lowercase());
        // Parents without values have no section of their own, so derive children from any descendant
        self.keys
            .iter()
            .filter(|(lower, _)| lower.starts_with(&prefix))
            .filter_map(|(_, (path, _))| path.get(prefix.len()..)?.split('\\').next())
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn host_path(&self, windows_path: &str) -> Option<PathBuf> {
        let drive = windows_path.get(..2)?.to_lowercase();
        if !drive.ends_with(':') {
            return None;
        }
        let rest = windows_path.get(2..)?.replace('\\', "/");
        let rest = rest.trim_start_matches('/');
        // Joined as one string: on Windows hosts `join("c:")` would be read as a drive
        Some(self.prefix.join(format!("dosdevices/{drive}/{rest}")))
    }
}

/// Parses `"Name"="value"` and `@="value"` lines; other value types are skipped.
fn parse_string_value(line: &str) -> Option<(String, String)> {
    let (name, rest) = if let Some(rest) = line.strip_prefix('@') {
        (String::new(), rest)
    } else {
        parse_quoted(line.strip_prefix('"')?)?
    };
    let rest = rest.strip_prefix('=')?;
    // REG_EXPAND_SZ is written as str(2):"..."
    let rest = rest.strip_prefix("str(2):").unwrap_or(rest);
    let (value, _) = parse_quoted(rest.strip_prefix('"')?)?;
    Some((name, value))
}

/// Reads a Wine-escaped string up to its closing quote, returning it and the remainder.
///
/// Mirrors Wine's `dump_strW`: C escapes, octal (up to 3 digits) for other control characters,
/// and `\x` with up to 4 hex digits per UTF-16 unit, so non-BMP characters arrive as surrogate pairs.
fn parse_quoted(s: &str) -> Option<(String, &str)> {
    let mut units: Vec<u16> = Vec::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let unit = match c {
            '"' => return Some((String::from_utf16_lossy(&units), &s[i + 1..])),
            '\\' => match chars.next()?.1 {
                'a' => 0x07,
                'b' => 0x08,
                't' => 0x09,
                'n' => 0x0a,
                'v' => 0x0b,
                'f' => 0x0c,
                'r' => 0x0d,
                'e' => 0x1b,
                'x' => take_digits(&mut chars, 0, 16, 4),
                digit @ '0'..='7' => {
                    let first = digit.to_digit(8).and_then(|d| u16::try_from(d).ok())?;
                    take_digits(&mut chars, first, 8, 2)
                }
                other => {
                    units.extend(other.encode_utf16(&mut [0; 2]).iter());
                    continue;
                }
            },
            c => {
                units.extend(c.encode_utf16(&mut [0; 2]).iter());
                continue;
            }
        };
        units.push(unit);
    }
    None
}

/// Consumes up to `max` digits in `radix`, accumulating onto `value`.
fn take_digits(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    mut value: u16,
    radix: u32,
    max: usize,
) -> u16 {
    for _ in 0..max {
        let Some(digit) = chars.peek().and_then(|&(_, c)| c.to_digit(radix)) else {
            break;
        };
        chars.next();
        // Both radixes fit a u16 within their digit limits
        value = value.wrapping_mul(radix as u16).wrapping_add(digit as u16);
    }
    value
}

fn unescape(s: &str) -> String {
    // Key names use the same escaping as values but have no surrounding quotes
    parse_quoted(&format!("{}\"", s.replace('"', "\\\"")))
        .map(|(v, _)| v)
        .unwrap_or_else(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYSTEM_REG: &str = r#"WINE REGISTRY Version 2
;; All keys relative to \\Machine

#arch=win64

[Software\\Classes\\origin2\\shell\\open\\command] 1700000000
#time=1da0000000000000
@="\"C:\\Program Files\\Electronic Arts\\EA Desktop\\EA Desktop\\EALauncher.exe\" \"%1\""

[Software\\Wow6432Node\\Ubisoft\\Launcher] 1700000000
"InstallDir"="C:/Program Files (x86)/Ubisoft/Ubisoft Game Launcher/"
"Timestamp"=dword:00000001

[Software\\Wow6432Node\\Ubisoft\\Launcher\\Installs\\635] 1700000000
"InstallDir"="C:/Games/Far Cry\xe9 5/"

[Software\\Wow6432Node\\Ubisoft\\Launcher\\Installs\\720\\Extra] 1700000000
"Other"="x"
"#;

    fn registry() -> WineRegistry {
        WineRegistry::parse(Path::new("/compat/123/pfx"), SYSTEM_REG)
    }

    #[test]
    fn reads_string_and_default_values() {
        let reg = registry();
        assert_eq!(
            reg.value(r"SOFTWARE\WOW6432Node\Ubisoft\Launcher", "installdir")
                .as_deref(),
            Some("C:/Program Files (x86)/Ubisoft/Ubisoft Game Launcher/")
        );
        assert_eq!(
            reg.value(r"SOFTWARE\Classes\origin2\shell\open\command", "")
                .as_deref(),
            Some(r#""C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EALauncher.exe" "%1""#)
        );
        assert_eq!(
            reg.value(r"SOFTWARE\WOW6432Node\Ubisoft\Launcher", "Timestamp"),
            None
        );
    }

    #[test]
    fn decodes_hex_escapes() {
        assert_eq!(
            registry()
                .value(
                    r"Software\Wow6432Node\Ubisoft\Launcher\Installs\635",
                    "InstallDir"
                )
                .as_deref(),
            Some("C:/Games/Far Cry\u{e9} 5/")
        );
    }

    fn unquote(escaped: &str) -> String {
        parse_quoted(&format!("{escaped}\"")).unwrap().0
    }

    #[test]
    fn decodes_every_wine_escape() {
        // Formats written by Wine's dump_strW
        assert_eq!(unquote(r"a\tb\nc\rd"), "a\tb\nc\rd");
        assert_eq!(unquote(r"\a\b\v\f\e"), "\u{7}\u{8}\u{b}\u{c}\u{1b}");
        assert_eq!(unquote(r#"q\"q\\"#), "q\"q\\");
        // Octal: short form, and zero-padded to 3 digits when a digit follows
        assert_eq!(unquote(r"\1x"), "\u{1}x");
        assert_eq!(unquote(r"\0017"), "\u{1}7");
        // Hex: short form, and zero-padded to 4 digits when a hex digit follows
        assert_eq!(unquote(r"\xe9!"), "\u{e9}!");
        assert_eq!(unquote(r"\x00e9a"), "\u{e9}a");
        // Characters outside the BMP are written as two UTF-16 units
        assert_eq!(unquote(r"\xd83d\xde00"), "\u{1f600}");
    }

    #[test]
    fn unterminated_string_is_rejected() {
        assert_eq!(parse_quoted("no closing quote"), None);
        assert_eq!(parse_quoted(r"trailing backslash\"), None);
    }

    #[test]
    fn lists_subkeys_including_ones_without_values() {
        assert_eq!(
            registry().subkeys(r"SOFTWARE\Wow6432Node\Ubisoft\Launcher\Installs"),
            ["635", "720"]
        );
    }

    #[test]
    fn maps_windows_paths_into_the_prefix() {
        let reg = registry();
        assert_eq!(
            reg.host_path(r"C:\Games\Title\"),
            Some(PathBuf::from("/compat/123/pfx/dosdevices/c:/Games/Title/"))
        );
        assert_eq!(
            reg.host_path("D:/Other"),
            Some(PathBuf::from("/compat/123/pfx/dosdevices/d:/Other"))
        );
        assert_eq!(reg.host_path("relative"), None);
        assert_eq!(reg.proton_compat_folder(), Some(Path::new("/compat/123")));
    }
}
