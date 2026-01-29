//! Update PR body with cover sheet.
//!
//! This command updates a GitHub PR's body by replacing the cover sheet block
//! (bounded by idempotent markers) with a freshly generated cover sheet from receipts.
//!
//! The update is **bounded**: only content between the markers is replaced.
//! Content outside the markers is preserved.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::pr_cover::{PrCoverArgs, replace_cover_sheet};

/// Arguments for pr-update command
#[derive(Debug, Clone, Default)]
pub struct PrUpdateArgs {
    /// PR number
    pub pr: u32,
    /// Directory containing receipts (default: .runs/pr/{pr}/latest/)
    pub run_dir: Option<PathBuf>,
    /// Description of what changed (optional)
    pub description: Option<String>,
    /// Save a copy to docs/audit/EXHIBITS/PR-{n}.md
    pub save_exhibit: bool,
    /// Dry run: show what would be updated without making changes
    pub dry_run: bool,
}

pub fn run(args: PrUpdateArgs) -> Result<()> {
    println!("{}", "Updating PR cover sheet...".blue().bold());

    // 1. Fetch current PR body
    println!("  Fetching PR #{} body...", args.pr);
    let current_body = fetch_pr_body(args.pr)?;

    // 2. Generate new cover sheet
    println!("  Generating new cover sheet from receipts...");
    let new_cover_sheet = generate_cover_sheet(&args, &current_body)?;

    // 3. Replace cover sheet in body (bounded update)
    let updated_body = replace_cover_sheet(&current_body, &new_cover_sheet);

    // Check if anything changed
    if updated_body == current_body {
        println!("{} PR body already up-to-date (no changes needed)", "OK".green());
        return Ok(());
    }

    if args.dry_run {
        println!("\n{}", "DRY RUN - would update PR body to:".yellow());
        println!("---");
        println!("{}", &updated_body);
        println!("---");
        return Ok(());
    }

    // 4. Update PR body
    println!("  Updating PR body...");
    update_pr_body(args.pr, &updated_body)?;
    println!("{} PR #{} body updated", "OK".green(), args.pr);

    // 5. Optionally save exhibit
    if args.save_exhibit {
        let exhibit_path = PathBuf::from(format!("docs/audit/EXHIBITS/PR-{}.md", args.pr));
        save_exhibit(&exhibit_path, &updated_body)?;
        println!("{} Exhibit saved to {}", "OK".green(), exhibit_path.display());
    }

    Ok(())
}

/// Fetch PR body using gh CLI
fn fetch_pr_body(pr: u32) -> Result<String> {
    let output = Command::new("gh")
        .args(["pr", "view", &pr.to_string(), "--json", "body", "--jq", ".body"])
        .output()
        .context("Failed to run gh pr view")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr view failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Generate cover sheet using pr_cover module
fn generate_cover_sheet(args: &PrUpdateArgs, current_body: &str) -> Result<String> {
    // Create a temp file to capture output
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("pr-cover-{}.md", args.pr));

    // Determine description: use arg if provided, otherwise try to extract from current body
    let description = if let Some(ref desc) = args.description {
        desc.clone()
    } else {
        extract_description_from_body(current_body)
            .unwrap_or_else(|| "Update PR with latest receipts".to_string())
    };

    // Run pr_cover::run with output to temp file
    let pr_cover_args = PrCoverArgs {
        pr: args.pr,
        run_dir: args.run_dir.clone(),
        output: Some(temp_file.clone()),
        description,
    };

    super::pr_cover::run(pr_cover_args)?;

    // Read the generated content
    let content = fs::read_to_string(&temp_file).with_context(|| {
        format!("Failed to read generated cover sheet from {}", temp_file.display())
    })?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    Ok(content)
}

fn extract_description_from_body(body: &str) -> Option<String> {
    // Try to find the "What changed" section and extract the first list item
    let marker = "### What changed";
    if let Some(pos) = body.find(marker) {
        let after_marker = &body[pos + marker.len()..];
        for line in after_marker.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                return Some(trimmed[1..].trim().to_string());
            }
        }
    }
    None
}

/// Update PR body using gh CLI
fn update_pr_body(pr: u32, body: &str) -> Result<()> {
    let output = Command::new("gh")
        .args(["pr", "edit", &pr.to_string(), "--body", body])
        .output()
        .context("Failed to run gh pr edit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gh pr edit failed: {}", stderr);
    }

    Ok(())
}

/// Save exhibit to docs/audit/EXHIBITS/
fn save_exhibit(path: &PathBuf, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Add frontmatter for exhibit
    let exhibit_content = format!(
        r#"---
id: EXHIBIT-PR-{}
title: "PR #{} Cover Sheet"
doc_type: exhibit
status: archived
generated_at: {}
---

{}
"#,
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").replace("PR-", ""),
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").replace("PR-", ""),
        chrono::Utc::now().to_rfc3339(),
        content
    );

    fs::write(path, exhibit_content)
        .with_context(|| format!("Failed to write exhibit to {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = PrUpdateArgs::default();
        assert_eq!(args.pr, 0);
        assert!(args.run_dir.is_none());
        assert!(!args.save_exhibit);
        assert!(!args.dry_run);
    }

    #[test]
    fn test_exhibit_path_format() {
        let path = PathBuf::from(format!("docs/audit/EXHIBITS/PR-{}.md", 123));
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "PR-123.md");
    }

    // Note: Integration tests for gh CLI would need mocking
    // These are left as manual tests
}
