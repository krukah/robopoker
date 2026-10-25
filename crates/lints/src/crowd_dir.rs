use std::path::Path;

use crate::*;

pub const ONE_FILE_DIRECTORY: &str = "one-file-directory";

impl Violation {
    /// A module directory holding a single `.rs` file adds a level the name of
    /// the file already carries. Fires on the lone file, so any PR that touches
    /// it answers for the directory. Cargo layout directories (`src`, `bin`,
    /// `tests`, `benches`, `examples`) are not module directories and never count.
    pub fn crowding(root: &Path, path: &str, src: &str) -> Option<Self> {
        if Language::for_path(path) != Some(Language::Rust) {
            return None;
        }
        if Directive::heads(src, ONE_FILE_DIRECTORY) {
            return None;
        }
        let full = root.join(path);
        let dir = full.parent()?;
        let file = full.file_name()?.to_str()?.to_string();
        let mut companions = 0usize;
        for entry in std::fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                return None;
            }
            if entry.file_name().to_string_lossy().ends_with(".rs") {
                companions += 1;
            }
        }
        if companions != 1 {
            return None;
        }
        let owner = dir.extension().is_none() && Self::sibling(dir);
        if !(owner || file == "mod.rs") {
            return None;
        }
        let name = dir.file_name()?.to_str()?.to_string();
        Some(Self::new(
            1,
            1,
            ONE_FILE_DIRECTORY,
            format!(
                "`{file}` is the only file in `{name}/`. Collapse the directory: move the file up beside `{name}.rs`, or fold a lone `mod.rs` into `{name}.rs`"
            ),
        ))
    }

    /// `foo/` is a module directory when `foo.rs` declares it from beside it.
    fn sibling(dir: &Path) -> bool {
        dir.parent()
            .map(|parent| parent.join(format!("{}.rs", dir.file_name().unwrap_or_default().to_string_lossy())))
            .as_deref()
            .is_some_and(Path::exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch;

    impl Scratch {
        fn dir(name: &str) -> std::path::PathBuf {
            let dir = std::env::temp_dir().join("lints-crowd-dir").join(name);
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            dir
        }
    }

    #[test]
    fn a_lone_file_in_a_module_directory_fails() {
        let root = Scratch::dir("lone");
        std::fs::create_dir_all(root.join("a/foo")).unwrap();
        std::fs::write(root.join("a/foo.rs"), "mod bar;\n").unwrap();
        std::fs::write(root.join("a/foo/bar.rs"), "pub struct Bar;\n").unwrap();
        assert!(Violation::crowding(&root, "a/foo/bar.rs", "pub struct Bar;\n").is_some());
    }

    #[test]
    fn a_lone_mod_rs_fails() {
        let root = Scratch::dir("modrs");
        std::fs::create_dir_all(root.join("a/foo")).unwrap();
        std::fs::write(root.join("a/foo/mod.rs"), "pub struct Foo;\n").unwrap();
        assert!(Violation::crowding(&root, "a/foo/mod.rs", "pub struct Foo;\n").is_some());
    }

    #[test]
    fn cargo_layout_directories_never_count() {
        let root = Scratch::dir("cargo");
        std::fs::create_dir_all(root.join("crate/src")).unwrap();
        std::fs::write(root.join("crate/src/lib.rs"), "\n").unwrap();
        assert!(Violation::crowding(&root, "crate/src/lib.rs", "\n").is_none());
    }

    #[test]
    fn a_second_file_or_a_subdirectory_settles_the_directory() {
        let root = Scratch::dir("full");
        std::fs::create_dir_all(root.join("a/foo")).unwrap();
        std::fs::write(root.join("a/foo.rs"), "mod bar;\nmod baz;\n").unwrap();
        std::fs::write(root.join("a/foo/bar.rs"), "\n").unwrap();
        std::fs::write(root.join("a/foo/baz.rs"), "\n").unwrap();
        assert!(Violation::crowding(&root, "a/foo/bar.rs", "\n").is_none());
        let root = Scratch::dir("deep");
        std::fs::create_dir_all(root.join("a/foo/inner")).unwrap();
        std::fs::write(root.join("a/foo.rs"), "mod bar;\n").unwrap();
        std::fs::write(root.join("a/foo/bar.rs"), "\n").unwrap();
        std::fs::write(root.join("a/foo/inner/x.rs"), "\n").unwrap();
        assert!(Violation::crowding(&root, "a/foo/bar.rs", "\n").is_none());
    }

    #[test]
    fn a_directive_on_the_first_line_exempts_the_file() {
        let root = Scratch::dir("allow");
        std::fs::create_dir_all(root.join("a/foo")).unwrap();
        std::fs::write(root.join("a/foo.rs"), "mod bar;\n").unwrap();
        let src = "// lint:allow(one-file-directory) generated bindings land here\npub struct Bar;\n";
        std::fs::write(root.join("a/foo/bar.rs"), src).unwrap();
        assert!(Violation::crowding(&root, "a/foo/bar.rs", src).is_none());
    }
}
