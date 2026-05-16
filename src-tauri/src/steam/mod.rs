pub mod apply;
pub mod artwork;
pub mod collections;
pub mod detect;
pub mod shortcuts;
pub mod sources;

pub fn non_steam_app_id(exe: &str, name: &str) -> u32 {
    crc32fast::hash(format!("{exe}{name}").as_bytes()) | 0x8000_0000
}
