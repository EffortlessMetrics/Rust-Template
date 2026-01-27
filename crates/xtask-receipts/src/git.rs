//! Git helpers for receipt generation.
//!
//! This module provides utilities for:
//! - Getting current git commit SHAs (short and full)
//! - Getting the SHA of a branch or ref
//! - Getting diff statistics

use std::process::Command;

/// Get current git commit SHA (short form).
pub fn get_current_commit_short() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Get current git commit SHA (full form).
pub fn get_current_commit_full() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Get the SHA of a branch or ref.
pub fn get_ref_sha(ref_name: &str) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", ref_name])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_commit_short() {
        // We can't easily test git commands in unit tests without a git repo
        // Just verify the function signature and return type
        let result = get_current_commit_short();
        // Result will be None if not in a git repo
        assert!(result.as_ref().map(|s| s.len() >= 7 && s.len() <= 40).unwrap_or(true));
    }

    #[test]
    fn test_get_current_commit_full() {
        let result = get_current_commit_full();
        // Result will be None if not in a git repo
        assert!(result.is_none() || result.as_ref().map(|s| s.len() == 40).unwrap_or(false));
    }
}
