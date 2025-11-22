use axum::Router;
use cucumber::World as CucumberWorld;
use http::HeaderMap;
use std::collections::HashMap;

/// Test world state - includes real HTTP router for integration testing
#[derive(Debug, CucumberWorld)]
pub struct World {
    /// Real HTTP router (in-process, no network)
    pub app: Router,
    /// Last HTTP response
    pub last_response: Option<Response>,
    /// Request headers to be sent with next request
    pub request_headers: HeaderMap,
    /// Keep temp dir alive
    pub _temp_dir: std::sync::Arc<tempfile::TempDir>,
    /// Last CLI command output (deprecated, use xtask_context)
    pub last_command_output: Option<CommandOutput>,
    /// XTask command execution context
    xtask_context: XtaskContext,
}

/// Context for xtask command execution
#[derive(Debug, Default)]
pub struct XtaskContext {
    pub last_command_output: Option<String>,
    pub last_command_status: Option<i32>,
    pub test_repo_path: Option<std::path::PathBuf>,
    pub env: HashMap<String, String>,
}

impl Default for World {
    fn default() -> Self {
        // Initialize telemetry for tests (idempotent)
        telemetry::init_tracing("acceptance-tests");

        let temp_dir = std::sync::Arc::new(tempfile::tempdir().expect("Failed to create temp dir"));
        let specs_dir = temp_dir.path().to_path_buf();
        let governance_repo =
            std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

        Self {
            app: app_http::app(governance_repo), // Real HTTP router from app-http crate
            last_response: None,
            request_headers: HeaderMap::new(),
            _temp_dir: temp_dir,
            last_command_output: None,
            xtask_context: XtaskContext::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: HeaderMap,
    /// Raw body text (for non-JSON responses like /metrics)
    pub raw_body: String,
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get immutable reference to xtask context
    pub fn xtask_context(&self) -> &XtaskContext {
        &self.xtask_context
    }

    /// Get mutable reference to xtask context
    pub fn xtask_context_mut(&mut self) -> &mut XtaskContext {
        &mut self.xtask_context
    }
}
