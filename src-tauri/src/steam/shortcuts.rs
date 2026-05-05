use crate::{
    error::{io_context, AppError, AppResult},
    models::ShortcutEntry,
    steam::non_steam_app_id,
};
use std::{fs, path::Path};

const TYPE_OBJECT: u8 = 0x00;
const TYPE_STRING: u8 = 0x01;
const TYPE_I32: u8 = 0x02;
const TYPE_END: u8 = 0x08;

pub fn read_shortcuts(path: &Path) -> AppResult<Vec<ShortcutEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = fs::read(path).map_err(io_context(path))?;
    parse_shortcuts(&bytes)
}

pub fn write_shortcuts(path: &Path, shortcuts: &[ShortcutEntry]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_context(parent))?;
    }

    let bytes = serialize_shortcuts(shortcuts);
    fs::write(path, bytes).map_err(io_context(path))
}

pub fn append_missing(existing: &mut Vec<ShortcutEntry>, additions: Vec<ShortcutEntry>) {
    for mut addition in additions {
        if addition.app_id == 0 {
            addition.app_id = non_steam_app_id(&addition.exe, &addition.app_name);
        }

        if let Some(item) = existing
            .iter_mut()
            .find(|item| same_shortcut_identity(item, &addition))
        {
            *item = addition;
            continue;
        }

        existing.push(addition);
    }
}

pub fn parse_shortcuts(bytes: &[u8]) -> AppResult<Vec<ShortcutEntry>> {
    let mut parser = Parser { bytes, index: 0 };
    let marker = parser.read_byte()?;
    if marker != TYPE_OBJECT {
        return Err(AppError::InvalidShortcuts(
            "missing root object marker".to_string(),
        ));
    }

    let root = parser.read_cstring()?;
    if root != "shortcuts" {
        return Err(AppError::InvalidShortcuts(
            "root object is not shortcuts".to_string(),
        ));
    }

    let mut shortcuts = Vec::new();
    loop {
        match parser.read_byte()? {
            TYPE_OBJECT => {
                let _index = parser.read_cstring()?;
                shortcuts.push(parser.read_shortcut()?);
            }
            TYPE_END => break,
            other => {
                return Err(AppError::InvalidShortcuts(format!(
                    "unexpected entry marker {other}"
                )));
            }
        }
    }

    Ok(shortcuts)
}

pub fn serialize_shortcuts(shortcuts: &[ShortcutEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    write_object_start(&mut out, "shortcuts");

    for (index, shortcut) in shortcuts.iter().enumerate() {
        write_object_start(&mut out, &index.to_string());
        write_i32(&mut out, "appid", shortcut.app_id);
        write_string(&mut out, "AppName", &shortcut.app_name);
        write_string(&mut out, "Exe", &shortcut.exe);
        write_string(&mut out, "StartDir", &shortcut.start_dir);
        write_string(&mut out, "icon", &shortcut.icon);
        write_string(&mut out, "ShortcutPath", &shortcut.shortcut_path);
        write_string(&mut out, "LaunchOptions", &shortcut.launch_options);
        write_i32(&mut out, "IsHidden", shortcut.is_hidden as u32);
        write_i32(
            &mut out,
            "AllowDesktopConfig",
            shortcut.allow_desktop_config as u32,
        );
        write_i32(&mut out, "AllowOverlay", shortcut.allow_overlay as u32);
        write_i32(&mut out, "openvr", shortcut.open_vr as u32);
        write_i32(&mut out, "Devkit", shortcut.devkit as u32);
        write_string(&mut out, "DevkitGameID", &shortcut.devkit_game_id);
        write_i32(&mut out, "DevkitOverrideAppID", 0);
        write_i32(&mut out, "LastPlayTime", shortcut.last_play_time);
        write_object_start(&mut out, "tags");
        for (tag_index, tag) in shortcut.tags.iter().enumerate() {
            write_string(&mut out, &tag_index.to_string(), tag);
        }
        out.push(TYPE_END);
        out.push(TYPE_END);
    }

    out.push(TYPE_END);
    out.push(TYPE_END);
    out
}

fn same_shortcut_identity(left: &ShortcutEntry, right: &ShortcutEntry) -> bool {
    left.app_name.eq_ignore_ascii_case(&right.app_name) && left.exe.eq_ignore_ascii_case(&right.exe)
}

fn write_object_start(out: &mut Vec<u8>, name: &str) {
    out.push(TYPE_OBJECT);
    out.extend_from_slice(name.as_bytes());
    out.push(0);
}

fn write_string(out: &mut Vec<u8>, key: &str, value: &str) {
    out.push(TYPE_STRING);
    out.extend_from_slice(key.as_bytes());
    out.push(0);
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

fn write_i32(out: &mut Vec<u8>, key: &str, value: u32) {
    out.push(TYPE_I32);
    out.extend_from_slice(key.as_bytes());
    out.push(0);
    out.extend_from_slice(&value.to_le_bytes());
}

struct Parser<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn read_shortcut(&mut self) -> AppResult<ShortcutEntry> {
        let mut shortcut = ShortcutEntry::default();

        loop {
            match self.read_byte()? {
                TYPE_STRING => {
                    let key = self.read_cstring()?;
                    let value = self.read_cstring()?;
                    match key.to_ascii_lowercase().as_str() {
                        "appname" => shortcut.app_name = value,
                        "exe" => shortcut.exe = value,
                        "startdir" => shortcut.start_dir = value,
                        "icon" => shortcut.icon = value,
                        "shortcutpath" => shortcut.shortcut_path = value,
                        "launchoptions" => shortcut.launch_options = value,
                        "devkitgameid" => shortcut.devkit_game_id = value,
                        _ => {}
                    }
                }
                TYPE_I32 => {
                    let key = self.read_cstring()?;
                    let value = self.read_u32()?;
                    match key.to_ascii_lowercase().as_str() {
                        "appid" => shortcut.app_id = value,
                        "ishidden" => shortcut.is_hidden = value != 0,
                        "allowdesktopconfig" => shortcut.allow_desktop_config = value != 0,
                        "allowoverlay" => shortcut.allow_overlay = value != 0,
                        "openvr" => shortcut.open_vr = value != 0,
                        "devkit" => shortcut.devkit = value != 0,
                        "lastplaytime" => shortcut.last_play_time = value,
                        _ => {}
                    }
                }
                TYPE_OBJECT => {
                    let key = self.read_cstring()?;
                    if key == "tags" {
                        shortcut.tags = self.read_tags()?;
                    } else {
                        self.skip_object()?;
                    }
                }
                TYPE_END => break,
                other => {
                    return Err(AppError::InvalidShortcuts(format!(
                        "unexpected shortcut marker {other}"
                    )));
                }
            }
        }

        Ok(shortcut)
    }

    fn read_tags(&mut self) -> AppResult<Vec<String>> {
        let mut tags = Vec::new();
        loop {
            match self.read_byte()? {
                TYPE_STRING => {
                    let _key = self.read_cstring()?;
                    tags.push(self.read_cstring()?);
                }
                TYPE_END => break,
                TYPE_OBJECT => {
                    let _key = self.read_cstring()?;
                    self.skip_object()?;
                }
                TYPE_I32 => {
                    let _key = self.read_cstring()?;
                    let _value = self.read_u32()?;
                }
                other => {
                    return Err(AppError::InvalidShortcuts(format!(
                        "unexpected tag marker {other}"
                    )));
                }
            }
        }
        Ok(tags)
    }

    fn skip_object(&mut self) -> AppResult<()> {
        loop {
            match self.read_byte()? {
                TYPE_STRING => {
                    let _key = self.read_cstring()?;
                    let _value = self.read_cstring()?;
                }
                TYPE_I32 => {
                    let _key = self.read_cstring()?;
                    let _value = self.read_u32()?;
                }
                TYPE_OBJECT => {
                    let _key = self.read_cstring()?;
                    self.skip_object()?;
                }
                TYPE_END => break,
                other => {
                    return Err(AppError::InvalidShortcuts(format!(
                        "unexpected nested marker {other}"
                    )));
                }
            }
        }
        Ok(())
    }

    fn read_byte(&mut self) -> AppResult<u8> {
        let byte = self
            .bytes
            .get(self.index)
            .copied()
            .ok_or_else(|| AppError::InvalidShortcuts("unexpected end of file".to_string()))?;
        self.index += 1;
        Ok(byte)
    }

    fn read_cstring(&mut self) -> AppResult<String> {
        let start = self.index;
        while let Some(byte) = self.bytes.get(self.index) {
            self.index += 1;
            if *byte == 0 {
                return Ok(String::from_utf8_lossy(&self.bytes[start..self.index - 1]).to_string());
            }
        }
        Err(AppError::InvalidShortcuts(
            "unterminated string".to_string(),
        ))
    }

    fn read_u32(&mut self) -> AppResult<u32> {
        let end = self.index + 4;
        let bytes = self
            .bytes
            .get(self.index..end)
            .ok_or_else(|| AppError::InvalidShortcuts("truncated integer".to_string()))?;
        self.index = end;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}
