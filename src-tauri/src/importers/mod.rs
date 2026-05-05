pub mod epic;
pub mod manual;
pub mod playnite;

use std::path::Path;

pub fn quote_path(path: &Path) -> String {
    format!("\"{}\"", path.display())
}
