use std::path::{Component, Path, PathBuf};

/// Appx logos can be stored as scale/targetsize-qualified files rather than the literal name.
pub(super) fn package_icon(root: &Path, logo: &str) -> Option<PathBuf> {
    let relative = Path::new(logo);
    if logo.is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let root_canonical = root.canonicalize().ok()?;
    let is_local_file = |path: &Path| {
        path.is_file()
            && path
                .canonicalize()
                .is_ok_and(|path| path.starts_with(&root_canonical))
    };
    let exact = root.join(relative);
    if is_local_file(&exact) {
        return Some(exact);
    }
    let prefix = format!("{}.", exact.file_stem()?.to_str()?);
    let extension = exact.extension()?;
    let mut variants = std::fs::read_dir(exact.parent()?)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension() == Some(extension)
                && path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&prefix))
                && is_local_file(path)
        })
        .collect::<Vec<_>>();
    variants.sort();
    variants.into_iter().next()
}

/// DisplayIcon can quote the path and append a resource index; only image files are copied.
pub(super) fn registry_icon(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let value = value
        .rsplit_once(',')
        .filter(|(_, index)| index.trim().parse::<i32>().is_ok())
        .map_or(value, |(path, _)| path);
    let path = PathBuf::from(value.trim().trim_matches('"'));
    let extension = path.extension()?.to_str()?;
    (path.is_file()
        && ["ico", "png", "jpg", "jpeg"]
            .iter()
            .any(|allowed| extension.eq_ignore_ascii_case(allowed)))
    .then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "fsa-icons-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(dir.join("Images")).unwrap();
            Self(dir)
        }
        fn file(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, b"fixture").unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn exact_and_qualified_package_logos() {
        let fixture = Fixture::new();
        let qualified = fixture.file("Images/Logo.scale-100.png");
        fixture.file("Images/OtherLogo.scale-100.png");
        assert_eq!(package_icon(&fixture.0, "Images/Logo.png"), Some(qualified));
        let exact = fixture.file("Images/Logo.png");
        assert_eq!(package_icon(&fixture.0, "Images/Logo.png"), Some(exact));
    }

    #[test]
    fn rejects_missing_and_non_package_paths() {
        let fixture = Fixture::new();
        for name in [
            "",
            "../Logo.png",
            "C:\\outside.png",
            "ms-resource:Logo",
            "Missing.png",
        ] {
            assert_eq!(package_icon(&fixture.0, name), None);
        }
    }

    #[test]
    fn accepts_registry_image_paths_not_executable_resources() {
        let fixture = Fixture::new();
        let icon = fixture.file("game icon.ico");
        assert_eq!(
            registry_icon(&format!("\"{}\",0", icon.display())),
            Some(icon)
        );
        let exe = fixture.file("game.exe");
        assert_eq!(registry_icon(&format!("{},1", exe.display())), None);
        assert_eq!(registry_icon("missing.ico"), None);
    }
}
