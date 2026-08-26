pub mod apply;
pub mod artwork;
pub mod collections;
pub mod detect;
pub mod plan;
#[cfg(unix)]
pub mod proton;
pub mod shortcuts;
pub mod sources;

pub fn non_steam_app_id(exe: &str, name: &str) -> u32 {
    crc32fast::hash(format!("{exe}{name}").as_bytes()) | 0x8000_0000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_forces_high_bit() {
        // CRC32("") = 0, so the | 0x8000_0000 is the only contributor to this value.
        assert_eq!(non_steam_app_id("", ""), 0x8000_0000);
    }

    #[test]
    fn high_bit_always_set() {
        assert!(non_steam_app_id("\"game.exe\"", "My Game") & 0x8000_0000 != 0);
        assert!(non_steam_app_id("\"C:\\Games\\app.exe\"", "A Title") & 0x8000_0000 != 0);
    }

    #[test]
    fn is_deterministic() {
        let a = non_steam_app_id("\"game.exe\"", "My Game");
        let b = non_steam_app_id("\"game.exe\"", "My Game");
        assert_eq!(a, b);
    }

    #[test]
    fn exe_and_name_both_affect_result() {
        let base = non_steam_app_id("\"game.exe\"", "My Game");
        assert_ne!(base, non_steam_app_id("\"other.exe\"", "My Game"));
        assert_ne!(base, non_steam_app_id("\"game.exe\"", "Other Game"));
    }

    #[test]
    fn concatenation_order_is_exe_then_name() {
        // Flipping the arguments must change the ID, proving the format is "{exe}{name}".
        assert_ne!(non_steam_app_id("ab", "cd"), non_steam_app_id("cd", "ab"));
    }
}
