//! GitHub CLI integration for issue tracking.
//!
//! This module provides a wrapper around the `gh` CLI for creating and
//! managing GitHub issues from friction and question entries.
//!
//! ## Future Work
//!
//! Some methods are marked `#[allow(dead_code)]` as they are implemented
//! for upcoming features:
//!
//! - `get_repo()` - For `--repo` override support
//! - `close_issue()` / `get_issue_state()` - For friction resolution sync
//! - `question_issue_body()` / `question_labels()` - For `question-gh-create`

use anyhow::{Context, Result};
use std::process::Command;

/// Reference to a GitHub issue
#[derive(Debug, Clone)]
pub struct IssueRef {
    pub number: u64,
    pub url: String,
}

impl IssueRef {
    /// Format as "#123" for storage in YAML
    pub fn as_short(&self) -> String {
        format!("#{}", self.number)
    }

    /// Parse from "#123" or "123" format
    pub fn parse(s: &str) -> Option<Self> {
        let num_str = s.trim_start_matches('#');
        let number = num_str.parse::<u64>().ok()?;
        Some(IssueRef { number, url: String::new() })
    }
}

/// GitHub CLI wrapper
pub struct GhClient;

impl GhClient {
    /// Check if gh CLI is installed and authenticated
    pub fn check_auth() -> Result<()> {
        // Check if gh is installed
        let which_output = Command::new("which").arg("gh").output();

        match which_output {
            Ok(output) if !output.status.success() => {
                anyhow::bail!(
                    "GitHub CLI (gh) not found. Install it from: https://cli.github.com/\n\
                     Then run: gh auth login"
                );
            }
            Err(_) => {
                anyhow::bail!(
                    "GitHub CLI (gh) not found. Install it from: https://cli.github.com/\n\
                     Then run: gh auth login"
                );
            }
            _ => {}
        }

        // Check if authenticated
        let auth_output = Command::new("gh")
            .args(["auth", "status"])
            .output()
            .context("Failed to run 'gh auth status'")?;

        if !auth_output.status.success() {
            let stderr = String::from_utf8_lossy(&auth_output.stderr);
            anyhow::bail!(
                "GitHub CLI not authenticated.\n\
                 Run: gh auth login\n\
                 Details: {}",
                stderr.trim()
            );
        }

        Ok(())
    }

    /// Get the current repository (owner/repo)
    #[allow(dead_code)]
    pub fn get_repo() -> Result<String> {
        let output = Command::new("gh")
            .args(["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"])
            .output()
            .context("Failed to run 'gh repo view'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Failed to get repository info. Are you in a git repo with a GitHub remote?\n\
                 Details: {}",
                stderr.trim()
            );
        }

        let repo = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if repo.is_empty() {
            anyhow::bail!("Could not determine repository name");
        }

        Ok(repo)
    }

    /// Create a GitHub issue
    ///
    /// Uses --body-file for safer handling of multi-line bodies and special characters.
    pub fn create_issue(title: &str, body: &str, labels: &[String]) -> Result<IssueRef> {
        Self::check_auth()?;

        // Write body to a temp file to avoid shell quoting issues
        let temp_dir = std::env::temp_dir();
        let body_file = temp_dir.join(format!("gh-issue-body-{}.md", std::process::id()));
        std::fs::write(&body_file, body).context("Failed to write issue body to temp file")?;

        // Ensure cleanup on all exit paths
        let _cleanup = scopeguard::guard(body_file.clone(), |path| {
            let _ = std::fs::remove_file(path);
        });

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "create", "--title", title, "--body-file"]);
        cmd.arg(&body_file);

        // Add non-empty labels (trim and dedupe)
        let mut seen_labels = std::collections::HashSet::new();
        for label in labels {
            let trimmed = label.trim();
            if !trimmed.is_empty() && seen_labels.insert(trimmed.to_string()) {
                cmd.args(["--label", trimmed]);
            }
        }

        let output = cmd.output().context("Failed to run 'gh issue create'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create issue: {}", stderr.trim());
        }

        // Parse the URL from stdout (gh issue create outputs the URL)
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Extract issue number from URL (e.g., https://github.com/owner/repo/issues/123)
        let number = url
            .rsplit('/')
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .context("Failed to parse issue number from URL")?;

        Ok(IssueRef { number, url })
    }

    /// Close a GitHub issue
    #[allow(dead_code)]
    pub fn close_issue(number: u64) -> Result<()> {
        Self::check_auth()?;

        let output = Command::new("gh")
            .args(["issue", "close", &number.to_string()])
            .output()
            .context("Failed to run 'gh issue close'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to close issue #{}: {}", number, stderr.trim());
        }

        Ok(())
    }

    /// Get issue state (open/closed)
    #[allow(dead_code)]
    pub fn get_issue_state(number: u64) -> Result<String> {
        Self::check_auth()?;

        let output = Command::new("gh")
            .args(["issue", "view", &number.to_string(), "--json", "state", "--jq", ".state"])
            .output()
            .context("Failed to run 'gh issue view'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get issue #{} state: {}", number, stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Open issue in browser
    pub fn open_in_browser(number: u64) -> Result<()> {
        let output = Command::new("gh")
            .args(["issue", "view", &number.to_string(), "--web"])
            .output()
            .context("Failed to run 'gh issue view --web'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to open issue #{} in browser: {}", number, stderr.trim());
        }

        Ok(())
    }
}

/// Generate issue body from a friction entry
#[allow(clippy::too_many_arguments)]
pub fn friction_issue_body(
    id: &str,
    summary: &str,
    description: &str,
    category: &str,
    severity: &str,
    date: &str,
    flow: Option<&str>,
    phase: Option<&str>,
    refs: &[String],
) -> String {
    let mut body = format!(
        "## Friction: {}\n\n\
         **ID**: `{}` | **Category**: {} | **Severity**: {} | **Date**: {}\n\n\
         ### Description\n\n\
         {}\n",
        summary, id, category, severity, date, description
    );

    if flow.is_some() || phase.is_some() {
        body.push_str("\n### Context\n\n");
        if let Some(f) = flow {
            body.push_str(&format!("- **Flow**: {}\n", f));
        }
        if let Some(p) = phase {
            body.push_str(&format!("- **Phase**: {}\n", p));
        }
    }

    if !refs.is_empty() {
        body.push_str(&format!("\n### Related\n\n- REQ/AC refs: {}\n", refs.join(", ")));
    }

    body.push_str(&format!(
        "\n---\n*Auto-created from friction entry `{}` via `cargo xtask friction-gh-create`*\n",
        id
    ));

    body
}

/// Generate issue body from a question entry
#[allow(dead_code, clippy::too_many_arguments)]
pub fn question_issue_body(
    id: &str,
    summary: &str,
    flow: &str,
    phase: &str,
    description: &str,
    created_by: &str,
    created_at: &str,
    options: &[(String, String)],         // (label, description)
    recommendation: Option<(&str, &str)>, // (option_label, rationale)
    refs: &[String],
) -> String {
    let mut body = format!(
        "## Question: {}\n\n\
         **ID**: `{}` | **Flow**: {} | **Phase**: {}\n\
         **Created by**: {} | **Created at**: {}\n\n\
         ### Context\n\n\
         {}\n",
        summary, id, flow, phase, created_by, created_at, description
    );

    if !options.is_empty() {
        body.push_str("\n### Options\n\n");
        for (label, desc) in options {
            body.push_str(&format!("- **{}**: {}\n", label, desc));
        }
    }

    if let Some((option_label, rationale)) = recommendation {
        body.push_str(&format!(
            "\n### Recommendation\n\n\
             **Recommended**: {}\n\n\
             {}\n",
            option_label, rationale
        ));
    }

    if !refs.is_empty() {
        body.push_str(&format!("\n### Related\n\n- REQ/AC refs: {}\n", refs.join(", ")));
    }

    body.push_str(&format!(
        "\n---\n*Auto-created from question `{}` via `cargo xtask question-gh-create`*\n",
        id
    ));

    body
}

/// Map friction severity to GitHub labels
pub fn friction_labels(category: &str, severity: &str) -> Vec<String> {
    let mut labels = vec!["friction".to_string(), format!("category:{}", category)];

    match severity {
        "critical" => labels.push("priority:critical".to_string()),
        "high" => labels.push("priority:high".to_string()),
        _ => {}
    }

    labels
}

/// Map question to GitHub labels
#[allow(dead_code)]
pub fn question_labels(flow: &str) -> Vec<String> {
    vec!["question".to_string(), "needs-decision".to_string(), format!("flow:{}", flow)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_ref_parse() {
        let ref1 = IssueRef::parse("#123").unwrap();
        assert_eq!(ref1.number, 123);

        let ref2 = IssueRef::parse("456").unwrap();
        assert_eq!(ref2.number, 456);

        assert!(IssueRef::parse("abc").is_none());
    }

    #[test]
    fn test_issue_ref_short() {
        let issue_ref = IssueRef { number: 42, url: "https://example.com".to_string() };
        assert_eq!(issue_ref.as_short(), "#42");
    }

    #[test]
    fn test_friction_labels() {
        let labels = friction_labels("tooling", "critical");
        assert!(labels.contains(&"friction".to_string()));
        assert!(labels.contains(&"category:tooling".to_string()));
        assert!(labels.contains(&"priority:critical".to_string()));
    }

    #[test]
    fn test_friction_issue_body() {
        let body = friction_issue_body(
            "FRICTION-TEST-001",
            "Test summary",
            "Test description",
            "tooling",
            "high",
            "2025-01-01",
            Some("bundle"),
            Some("selection"),
            &["REQ-001".to_string()],
        );

        assert!(body.contains("FRICTION-TEST-001"));
        assert!(body.contains("Test summary"));
        assert!(body.contains("tooling"));
        assert!(body.contains("Flow"));
        assert!(body.contains("REQ-001"));
    }
}
