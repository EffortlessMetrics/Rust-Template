//! Path-safety helpers for manifest key validation.

use std::path::{Component, Path};

/// Returns `true` when `path` is a safe relative path suitable for use as a
/// manifest key.
///
/// Rejects absolute paths, parent-directory traversals (`..`), and Windows
/// drive prefixes.
pub fn is_safe_relative_path(path: &Path) -> bool {
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            // ParentDir (..), RootDir (/), Prefix (C:\) are all unsafe
            _ => return false,
        }
    }
    // Empty path is not useful
    path.components().next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn safe_paths_accepted() {
        assert!(is_safe_relative_path(Path::new("foo/bar.txt")));
        assert!(is_safe_relative_path(Path::new("specs/spec_ledger.yaml")));
        assert!(is_safe_relative_path(Path::new("a")));
    }

    #[test]
    fn parent_dir_rejected() {
        assert!(!is_safe_relative_path(Path::new("../etc/passwd")));
        assert!(!is_safe_relative_path(Path::new("foo/../../bar")));
    }

    #[test]
    fn absolute_paths_rejected() {
        assert!(!is_safe_relative_path(Path::new("/etc/passwd")));
        assert!(!is_safe_relative_path(Path::new("/foo/bar")));
    }

    #[test]
    fn empty_path_rejected() {
        assert!(!is_safe_relative_path(Path::new("")));
    }
}
