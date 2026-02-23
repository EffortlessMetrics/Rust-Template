//! Process-global state guards for safe env var and cwd manipulation.
//!
//! These guards are designed for test code that needs to temporarily modify
//! process-global state (environment variables, current working directory).
//!
//! The guards are reentrant-safe: nested guards in the same thread work correctly,
//! with inner guards snapshotting and restoring "current" (possibly already mutated)
//! state, and outer guards restoring original state.

// This module is the canonical implementation of process-global state guards.
// It must use the disallowed methods to provide safe wrappers for them.
#![allow(clippy::disallowed_methods)]

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Global lock serializing all process-global state modifications.
///
/// This is a reentrant mutex to support nested guard patterns:
/// - Inner guard snapshots "current" state (possibly already mutated by outer)
/// - Inner drop restores to that "current" state
/// - Outer drop restores to original state
///
/// This single lock covers both env vars and cwd to prevent any possibility
/// of concurrent modification of process-global state.
static PROCESS_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

fn lock() -> ReentrantMutexGuard<'static, ()> {
    PROCESS_LOCK.get_or_init(|| ReentrantMutex::new(())).lock()
}

/// RAII guard for process environment variables.
///
/// Serializes env mutation via a global lock and restores the observed
/// pre-test state on Drop (panic-safe).
///
/// # Example
///
/// ```no_run
/// use testing::process::EnvVarGuard;
///
/// #[test]
/// fn test_env_var() {
///     let guard = EnvVarGuard::new(&["RUST_LOG", "MY_VAR"]);
///     guard.set("RUST_LOG", "debug");
///     guard.remove("MY_VAR");
///
///     // Test code here...
///
/// } // Environment restored automatically
/// ```
pub struct EnvVarGuard {
    _lock: ReentrantMutexGuard<'static, ()>,
    snapshot: Vec<(&'static str, Option<String>)>,
}

impl EnvVarGuard {
    /// Create a new guard that snapshots the specified environment variables.
    ///
    /// The guard holds a global lock for the duration of its lifetime,
    /// preventing concurrent env var modifications.
    pub fn new(keys: &[&'static str]) -> Self {
        let _lock = lock();
        let snapshot = keys.iter().copied().map(|k| (k, std::env::var(k).ok())).collect();
        Self { _lock, snapshot }
    }

    /// Set an environment variable.
    ///
    /// The global process lock is held for the duration of this guard's lifetime,
    /// preventing concurrent modifications. The value will be restored on Drop.
    pub fn set(&self, key: &'static str, value: &str) {
        // SAFETY: env mutation is process-global; we serialize via PROCESS_LOCK
        // and restore on Drop.
        unsafe { std::env::set_var(key, value) };
    }

    /// Remove an environment variable.
    ///
    /// The global process lock is held for the duration of this guard's lifetime,
    /// preventing concurrent modifications. The value will be restored on Drop.
    pub fn remove(&self, key: &'static str) {
        // SAFETY: env mutation is process-global; we serialize via PROCESS_LOCK
        // and restore on Drop.
        unsafe { std::env::remove_var(key) };
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        // Restore exactly what we observed before the guard was created.
        for (key, value) in self.snapshot.iter() {
            // SAFETY: restoring pre-test state while holding PROCESS_LOCK.
            unsafe {
                match value {
                    Some(v) => std::env::set_var(*key, v),
                    None => std::env::remove_var(*key),
                }
            }
        }
    }
}

/// RAII guard for current working directory.
///
/// Serializes cwd mutation via a global lock and restores the original
/// directory on Drop (panic-safe).
///
/// # Example
///
/// ```no_run
/// use testing::process::CwdGuard;
/// use std::path::Path;
///
/// #[test]
/// fn test_in_temp_dir() {
///     let temp_dir = std::env::temp_dir();
///     let guard = CwdGuard::chdir(&temp_dir);
///
///     // Test code here runs in temp_dir...
///
/// } // Original working directory restored automatically
/// ```
pub struct CwdGuard {
    _lock: ReentrantMutexGuard<'static, ()>,
    original: PathBuf,
}

impl CwdGuard {
    /// Change to a new directory and return a guard that restores the original on drop.
    ///
    /// # Panics
    ///
    /// Panics if the current directory cannot be read or if changing to the new
    /// directory fails. This is acceptable in test code.
    pub fn chdir(new_dir: &Path) -> Self {
        let _lock = lock();
        let original = std::env::current_dir().expect("current_dir should be readable");
        std::env::set_current_dir(new_dir).expect("set_current_dir should succeed in tests");
        Self { _lock, original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        // Never panic in Drop - silently restore if possible.
        let _ = std::env::set_current_dir(&self.original);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn env_var_guard_restores_on_drop() {
        let test_key = "TESTING_CRATE_TEST_VAR";

        // Ensure clean state
        let original = env::var(test_key).ok();

        {
            let guard = EnvVarGuard::new(&[test_key]);
            guard.set(test_key, "test-value");
            assert_eq!(env::var(test_key).ok(), Some("test-value".to_string()));
        }

        // Should be restored
        assert_eq!(env::var(test_key).ok(), original);
    }

    #[test]
    fn env_var_guard_restores_removed_var() {
        let test_key = "TESTING_CRATE_REMOVE_VAR";

        // Set initial value using a guard (reentrant lock allows nesting)
        let setup_guard = EnvVarGuard::new(&[test_key]);
        setup_guard.set(test_key, "initial");

        {
            let guard = EnvVarGuard::new(&[test_key]);
            guard.remove(test_key);
            assert!(env::var(test_key).is_err());
        }

        // Should be restored to "initial" (set by outer guard)
        assert_eq!(env::var(test_key).ok(), Some("initial".to_string()));

        // Cleanup: outer guard restores original state on drop
        drop(setup_guard);
    }

    #[test]
    fn cwd_guard_restores_on_drop() {
        let original_dir = env::current_dir().unwrap();
        let temp_dir = env::temp_dir();

        // Use canonicalize to resolve symlinks (e.g. /tmp -> /private/tmp on macOS)
        let canonical_temp = temp_dir.canonicalize().unwrap_or(temp_dir.clone());

        {
            let _guard = CwdGuard::chdir(&temp_dir);
            let current = env::current_dir().unwrap();
            let canonical_current = current.canonicalize().unwrap_or(current);
            assert_eq!(canonical_current, canonical_temp);
        }

        // Should be restored
        assert_eq!(env::current_dir().unwrap(), original_dir);
    }
}
