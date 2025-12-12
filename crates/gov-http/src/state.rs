//! Platform state trait for dependency injection.

use gov_model::{GovernanceRepository, RepoContext};
use std::sync::Arc;

/// Trait that services implement to provide platform state.
///
/// This abstracts the dependencies needed by platform handlers.
pub trait PlatformState: Send + Sync {
    /// Get the repository context.
    fn context(&self) -> &RepoContext;

    /// Get the governance repository.
    fn governance_repo(&self) -> Arc<dyn GovernanceRepository>;
}

/// Default platform state implementation.
#[derive(Clone)]
pub struct DefaultPlatformState {
    context: RepoContext,
    repo: Arc<dyn GovernanceRepository>,
}

impl DefaultPlatformState {
    /// Create new default state.
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
