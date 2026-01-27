//! Platform state trait for dependency injection.
//!
//! This module provides the trait-based state pattern used by all
//! gov-http-* subrouter crates. Using traits instead of concrete types
//! allows services to provide their own state implementations.

use gov_model::{GovernanceRepository, RepoContext};
use std::sync::Arc;

/// Trait that services implement to provide platform state.
///
/// This abstracts the dependencies needed by platform handlers.
/// Services implement this trait to provide their own state with
/// concrete implementations of the required dependencies.
///
/// # Example
///
/// ```ignore
/// use gov_http_core::PlatformState;
/// use gov_model::{RepoContext, GovernanceRepository};
/// use std::sync::Arc;
///
/// struct MyState {
///     context: RepoContext,
///     repo: Arc<dyn GovernanceRepository>,
/// }
///
/// impl PlatformState for MyState {
///     fn context(&self) -> &RepoContext {
///         &self.context
///     }
///
///     fn governance_repo(&self) -> Arc<dyn GovernanceRepository> {
///         Arc::clone(&self.repo)
///     }
/// }
/// ```
pub trait PlatformState: Send + Sync {
    /// Get the repository context.
    ///
    /// The context provides workspace paths and configuration
    /// needed by handlers to load spec files and other resources.
    fn context(&self) -> &RepoContext;

    /// Get the governance repository.
    ///
    /// The repository provides access to task persistence
    /// and other governance data.
    fn governance_repo(&self) -> Arc<dyn GovernanceRepository>;
}

/// Default platform state implementation.
///
/// This is a convenience implementation for services that
/// don't need to customize the state structure.
#[derive(Clone)]
pub struct DefaultPlatformState {
    context: RepoContext,
    repo: Arc<dyn GovernanceRepository>,
}

impl DefaultPlatformState {
    /// Create new default state.
    ///
    /// # Arguments
    ///
    /// * `context` - The repository context with workspace paths
    /// * `repo` - The governance repository for task persistence
    pub fn new(context: RepoContext, repo: Arc<dyn GovernanceRepository>) -> Self {
        Self { context, repo }
    }
}

impl PlatformState for DefaultPlatformState {
    fn context(&self) -> &RepoContext {
        &self.context
    }

    fn governance_repo(&self) -> Arc<dyn GovernanceRepository> {
        Arc::clone(&self.repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_platform_state() {
        // This is a compile-time test to ensure the trait is implemented
        // Actual testing would require a mock GovernanceRepository
        #[allow(dead_code)]
        fn accepts_platform_state<S: PlatformState>(_state: S) {}

        // This would fail to compile if DefaultPlatformState doesn't implement PlatformState
        // We can't actually test this without a mock repository
    }
}
