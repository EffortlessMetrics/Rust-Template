use crate::{
    CorsConfig, PlatformAuthConfig, SecurityHeadersConfig, config::load_valid_config,
    resolve_workspace_root, security,
};
use business_core::governance::GovernanceRepository;
use gov_model::RepoContext;
use http_platform::PlatformState as HttpPlatformState;
use http_tasks::TasksState;
use http_todos::TodosStateTrait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Application state combining all required state for the HTTP layer.
///
/// This state implements all the required traits from the http-* crates
/// to enable seamless integration.
#[derive(Clone)]
pub struct AppState {
    /// Governance repository
    pub governance_repo: Arc<dyn GovernanceRepository>,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Validated configuration
    pub config: Option<spec_runtime::ValidatedConfig>,
    /// Platform authentication configuration
    pub platform_auth: PlatformAuthConfig,
    /// CORS configuration
    pub cors_config: CorsConfig,
    /// Security headers configuration
    pub security_headers_config: SecurityHeadersConfig,
    /// Repository context for gov-http integration
    pub repo_context: RepoContext,
}

// Implement http-core AppState trait
impl http_core::AppState for AppState {
    fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    fn governance_repo(&self) -> Arc<dyn business_core::governance::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }

    fn repo_context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn config(&self) -> Option<&spec_runtime::ValidatedConfig> {
        self.config.as_ref()
    }
}

// Implement gov-http PlatformState trait (for governance endpoints)
impl gov_http::PlatformState for AppState {
    fn context(&self) -> &RepoContext {
        &self.repo_context
    }

    fn governance_repo(&self) -> Arc<dyn gov_model::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

// Implement http-platform PlatformState trait
impl HttpPlatformState for AppState {
    fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    fn config(&self) -> Option<&spec_runtime::ValidatedConfig> {
        self.config.as_ref()
    }

    fn platform_auth(&self) -> &dyn http_platform::PlatformAuthConfig {
        &self.platform_auth
    }
}

// Implement http-tasks TasksState trait
impl TasksState for AppState {
    fn governance_repo(
        &self,
    ) -> std::sync::Arc<dyn business_core::governance::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

// Implement http-todos TodosStateTrait trait
impl TodosStateTrait for AppState {
    fn todos_state(&self) -> http_todos::TodosState {
        http_todos::TodosState::new()
    }
}

// Implement http-agents AgentsState trait
impl http_agents::AgentsState for AppState {
    fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    fn governance_repo(
        &self,
    ) -> std::sync::Arc<dyn business_core::governance::GovernanceRepository> {
        Arc::clone(&self.governance_repo)
    }
}

impl AppState {
    /// Create a new AppState with default configuration.
    ///
    /// Uses the default workspace root resolution. For tests or custom configurations,
    /// prefer `with_config()` which allows explicit workspace root specification.
    pub fn new(governance_repo: Arc<dyn GovernanceRepository>) -> Result<Self, String> {
        let workspace_root = resolve_workspace_root();
        Self::with_config(governance_repo, workspace_root, None)
    }

    pub fn with_config(
        governance_repo: Arc<dyn GovernanceRepository>,
        workspace_root: PathBuf,
        config: Option<spec_runtime::ValidatedConfig>,
    ) -> Result<Self, String> {
        let config = config.or_else(|| load_valid_config(&workspace_root));
        let platform_auth = security::PlatformAuthConfig::try_from_sources(config.as_ref())?;
        platform_auth.warn_if_misconfigured();

        // Initialize security configurations
        let cors_config = CorsConfig::from_sources(config.as_ref());
        let security_headers_config = SecurityHeadersConfig::from_sources(config.as_ref());

        // Create RepoContext for gov-http integration
        let repo_context = RepoContext::new(&workspace_root);

        Ok(Self {
            governance_repo,
            workspace_root,
            config,
            platform_auth,
            cors_config,
            security_headers_config,
            repo_context,
        })
    }
}
