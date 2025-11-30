use axum::Router;
use cucumber::World as CucumberWorld;
use http::HeaderMap;
use std::{collections::HashMap, fs, path::Path};

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
    /// Result of the most recent config validation
    pub config_validation_ok: Option<bool>,
    /// Error message from config validation (if any)
    pub config_validation_error: Option<String>,
    /// XTask command execution context
    xtask_context: XtaskContext,
    /// CLI command exit code
    pub cli_exit_code: Option<i32>,
    /// CLI command stdout
    pub cli_stdout: String,
    /// CLI command stderr
    pub cli_stderr: String,
    /// Parsed JSON output from CLI command
    pub cli_json_output: Option<serde_json::Value>,
}

/// Context for xtask command execution
#[derive(Debug, Default)]
pub struct XtaskContext {
    pub last_command_output: Option<String>,
    pub last_command_status: Option<i32>,
    pub test_repo_path: Option<std::path::PathBuf>,
    pub env: HashMap<String, String>,
    /// Path to test ADR file for cleanup (AC-PLT-004)
    pub test_adr_path: Option<std::path::PathBuf>,
    /// Path to backup file for cleanup (AC-PLT-009, AC-PLT-010)
    pub test_backup_path: Option<std::path::PathBuf>,
    /// Path to skills directory for validation (AC-TPL-AGENT-SKILLS)
    pub test_skills_dir: Option<std::path::PathBuf>,
    /// Path to release evidence generated during tests
    pub test_evidence_path: Option<std::path::PathBuf>,
}

impl Default for World {
    fn default() -> Self {
        // Initialize telemetry for tests (idempotent)
        telemetry::init_tracing("acceptance-tests");

        let temp_dir = std::sync::Arc::new(tempfile::tempdir().expect("Failed to create temp dir"));
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).expect("Failed to create temp specs directory");

        // Reset auth-related env vars so scenarios start from a known state
        // SAFETY: Test runner is single-threaded here; resetting env vars is scoped to the test process.
        unsafe {
            std::env::remove_var("PLATFORM_AUTH_MODE");
            std::env::remove_var("PLATFORM_AUTH_TOKEN");
        }

        // Seed temp specs with a copy of the workspace specs so endpoints can operate
        let workspace_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let workspace_specs = workspace_root.join("specs");
        copy_dir_recursive(&workspace_specs, &specs_dir)
            .expect("Failed to copy workspace specs into temp dir");

        // Copy config directory so startup validation succeeds in tests
        let workspace_config = workspace_root.join("config");
        let temp_config = temp_dir.path().join("config");
        if workspace_config.exists() {
            copy_dir_recursive(&workspace_config, &temp_config)
                .expect("Failed to copy workspace config into temp dir");
        } else {
            std::fs::create_dir_all(&temp_config).expect("Failed to create temp config directory");
        }

        // Copy README.md and CLAUDE.md to temp dir for test isolation (AC-PLT-013)
        let readme_src = workspace_root.join("README.md");
        let readme_dst = temp_dir.path().join("README.md");
        if readme_src.exists() {
            fs::copy(&readme_src, &readme_dst).expect("Failed to copy README.md to temp dir");
        }

        let claude_src = workspace_root.join("CLAUDE.md");
        let claude_dst = temp_dir.path().join("CLAUDE.md");
        if claude_src.exists() {
            fs::copy(&claude_src, &claude_dst).expect("Failed to copy CLAUDE.md to temp dir");
        }

        // Copy docs directory for tests that check for documentation files
        let workspace_docs = workspace_root.join("docs");
        let temp_docs = temp_dir.path().join("docs");
        if workspace_docs.exists() {
            copy_dir_recursive(&workspace_docs, &temp_docs)
                .expect("Failed to copy workspace docs into temp dir");
        }

        // Copy docker-compose.yaml for infrastructure tests
        let docker_compose_src = workspace_root.join("docker-compose.yaml");
        let docker_compose_dst = temp_dir.path().join("docker-compose.yaml");
        if docker_compose_src.exists() {
            fs::copy(&docker_compose_src, &docker_compose_dst)
                .expect("Failed to copy docker-compose.yaml to temp dir");
        }

        // Copy questions directory for platform questions API tests
        let workspace_questions = workspace_root.join("questions");
        let temp_questions = temp_dir.path().join("questions");
        if workspace_questions.exists() {
            copy_dir_recursive(&workspace_questions, &temp_questions)
                .expect("Failed to copy workspace questions into temp dir");
        }

        // Copy friction directory for platform friction API tests
        let workspace_friction = workspace_root.join("friction");
        let temp_friction = temp_dir.path().join("friction");
        if workspace_friction.exists() {
            copy_dir_recursive(&workspace_friction, &temp_friction)
                .expect("Failed to copy workspace friction into temp dir");
        }

        // Copy forks directory for platform forks API tests
        let workspace_forks = workspace_root.join("forks");
        let temp_forks = temp_dir.path().join("forks");
        if workspace_forks.exists() {
            copy_dir_recursive(&workspace_forks, &temp_forks)
                .expect("Failed to copy workspace forks into temp dir");
        }

        // Make the spec root discoverable for app-http/xtask consumers.
        // SAFETY: Updating process environment here is confined to the test runner setup.
        unsafe {
            std::env::set_var("SPEC_ROOT", temp_dir.path());
        }

        let governance_repo =
            std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

        Self {
            app: app_http::app(governance_repo), // Real HTTP router from app-http crate
            last_response: None,
            request_headers: HeaderMap::new(),
            _temp_dir: temp_dir,
            last_command_output: None,
            config_validation_ok: None,
            config_validation_error: None,
            xtask_context: XtaskContext::default(),
            cli_exit_code: None,
            cli_stdout: String::new(),
            cli_stderr: String::new(),
            cli_json_output: None,
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

    /// Rebuild the application router after mutating environment variables.
    pub fn reload_app(&mut self) {
        let specs_dir = self._temp_dir.path().join("specs");
        let governance_repo =
            std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));
        self.app =
            app_http::app_with_workspace_root(governance_repo, self._temp_dir.path().to_path_buf());
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), dest_path)?;
        }
    }

    Ok(())
}
