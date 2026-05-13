//! PR-scoped RIPR evidence command wrappers.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";
const IMPACTED_DIR: &str = "target/xtask/impacted-evidence";

/// Run or check `ripr check` PR evidence output.
pub fn pr(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let json_path = workspace_root.join(RIPR_PR_DIR).join("repo-exposure.json");
    let md_path = workspace_root.join(RIPR_PR_DIR).join("repo-exposure.md");

    if check {
        verify_json_file(&json_path)?;
        verify_nonempty_file(&md_path)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    fs::create_dir_all(workspace_root.join(RIPR_PR_DIR))
        .context("failed to create target/ripr/pr")?;
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--format")
        .arg("repo-exposure")
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        bail!("{ripr_bin} PR evidence failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    if !md_path.exists() {
        let markdown = String::from_utf8_lossy(&output.stdout);
        let content = if markdown.trim().is_empty() {
            "# RIPR PR Evidence\n\nRIPR produced JSON evidence.\n".to_string()
        } else {
            markdown.into_owned()
        };
        fs::write(&md_path, content)
            .with_context(|| format!("failed to write {}", md_path.display()))?;
    }

    pr(true)
}

/// Run or check `ripr review-comments` output.
pub fn review_comments(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let json_path = workspace_root.join(RIPR_REVIEW_DIR).join("comments.json");
    let md_path = workspace_root.join(RIPR_REVIEW_DIR).join("comments.md");

    if check {
        verify_json_file(&json_path)?;
        verify_nonempty_file(&md_path)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    fs::create_dir_all(workspace_root.join(RIPR_REVIEW_DIR))
        .context("failed to create target/ripr/review")?;
    let base = std::env::var("RIPR_BASE").unwrap_or_else(|_| "origin/main".to_string());
    let head = std::env::var("RIPR_HEAD").unwrap_or_else(|_| "HEAD".to_string());
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&base)
        .arg("--head")
        .arg(&head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        bail!("{ripr_bin} review-comments failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    if !md_path.exists() {
        fs::write(
            &md_path,
            "# RIPR Review Guidance\n\nNo Markdown review guidance was produced.\n",
        )
        .with_context(|| format!("failed to write {}", md_path.display()))?;
    }

    review_comments(true)
}

/// Produce a compact impacted-evidence receipt consumed by CI routing.
pub fn impacted_evidence() -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let out_dir = workspace_root.join(IMPACTED_DIR);
    fs::create_dir_all(&out_dir).context("failed to create impacted-evidence directory")?;

    let report = ImpactedEvidence {
        requires_targeted_mutation: false,
        reason: "fast evidence only: no impacted-evidence router is configured for this template"
            .to_string(),
        ripr: RiprImpact { requires_targeted_evidence: false },
    };

    fs::write(out_dir.join("latest.json"), format!("{}\n", serde_json::to_string_pretty(&report)?))
        .context("failed to write impacted evidence JSON")?;
    fs::write(
        out_dir.join("latest.md"),
        "# Impacted Evidence\n\n- Targeted mutation required: `false`\n- Reason: fast evidence only; no impacted-evidence router is configured for this template.\n",
    )
    .context("failed to write impacted evidence Markdown")?;

    println!("impacted-evidence: wrote target/xtask/impacted-evidence/latest.json");
    Ok(())
}

/// Route cargo-mutants for changed code when explicitly requested.
pub fn mutants_pr(changed: bool, full_owner: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "mutants-pr: dry-run changed={} full_owner={} (no mutation executed)",
            changed, full_owner
        );
        return Ok(());
    }

    let workspace_root = workspace_root_path()?;
    let mut command = Command::new("cargo");
    command.arg("mutants");
    if changed {
        command.arg("--in-diff").arg("origin/main");
    }
    if full_owner {
        command.arg("--package").arg("*");
    }
    let status =
        command.current_dir(&workspace_root).status().context("failed to run cargo mutants")?;
    if !status.success() {
        bail!("cargo mutants failed");
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ImpactedEvidence {
    requires_targeted_mutation: bool,
    reason: String,
    ripr: RiprImpact,
}

#[derive(Debug, Serialize, Deserialize)]
struct RiprImpact {
    requires_targeted_evidence: bool,
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn workspace_root_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("could not determine workspace root from CARGO_MANIFEST_DIR")
}

fn verify_json_file(path: &Path) -> Result<()> {
    let bytes =
        fs::read(path).with_context(|| format!("missing required file {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    if value.is_null() {
        bail!("{} must not contain null JSON", path.display());
    }
    Ok(())
}

fn verify_nonempty_file(path: &Path) -> Result<()> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("missing required file {}", path.display()))?;
    if contents.trim().is_empty() {
        bail!("{} must not be empty", path.display());
    }
    Ok(())
}
