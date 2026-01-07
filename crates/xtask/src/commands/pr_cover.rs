//! Generate PR cover sheet from receipts.
//!
//! This command generates a markdown cover sheet for PRs based on governance
//! receipts from a run directory. The cover sheet follows the canonical format
//! defined in `docs/audit/PR_COVER_SHEET.md` and includes:
//!
//! - What changed section
//! - Review map (where to look)
//! - Proof (receipts from selftest, gate checks)
//! - Errata section (what was wrong, how detected, how fixed)
//! - Unified budget (DevLT, compute spend)
//! - Reproduce locally instructions
//! - Machine-updateable swarm-meta block
//!
//! The output uses idempotent markers for safe re-generation.

use anyhow::{Context, Result};
use colored::Colorize;
use gov_receipts::{EconomicsReceipt, GateReceipt, GateStatus};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Idempotent marker: start of cover sheet block
const COVER_SHEET_START: &str = "<!-- pr-cover-sheet:start -->";
/// Idempotent marker: end of cover sheet block
const COVER_SHEET_END: &str = "<!-- pr-cover-sheet:end -->";

/// Arguments for pr-cover command
#[derive(Debug, Clone, Default)]
pub struct PrCoverArgs {
    /// PR number
    pub pr: u32,
    /// Directory containing receipts (default: .runs/pr/{pr}/latest/)
    pub run_dir: Option<PathBuf>,
    /// Output file (default: stdout)
    pub output: Option<PathBuf>,
    /// Description of what changed (optional, defaults to placeholder)
    pub description: Option<String>,
}

/// Errata entry structure matching the canonical format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrataEntry {
    /// What was incorrect
    pub wrong: String,
    /// How it was detected (gate name, reviewer, etc.)
    pub detected_by: String,
    /// Link to detecting gate/comment
    pub detected_link: Option<String>,
    /// Commit/PR that fixed it
    pub fix_commit: Option<String>,
    /// What prevention was added (test, gate)
    pub prevention: Option<String>,
    /// Link to prevention
    pub prevention_link: Option<String>,
}

/// Swarm metadata block (machine-updated, idempotent)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SwarmMeta {
    run_id: String,
    pr: u32,
    commit: String,
    receipts: ReceiptPaths,
    devlt_minutes: DevLtMeta,
    compute: ComputeMeta,
    generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReceiptPaths {
    gate: Option<String>,
    economics: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DevLtMeta {
    author: Option<String>,
    review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ComputeMeta {
    tokens_usd: Option<String>,
}

pub fn run(args: PrCoverArgs) -> Result<()> {
    println!("{}", "Generating PR cover sheet...".blue().bold());

    // Determine run directory
    let run_dir =
        args.run_dir.unwrap_or_else(|| PathBuf::from(format!(".runs/pr/{}/latest", args.pr)));

    // Check if receipts exist (all receipts are under receipts/ subdirectory)
    let gate_path = run_dir.join("receipts/gate.json");
    let economics_path = run_dir.join("receipts/economics.json");

    // Try to load gate receipt using gov-receipts types
    let gate_receipt: Option<GateReceipt> = if gate_path.exists() {
        match fs::read_to_string(&gate_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Try to load economics receipt using gov-receipts types
    let economics: Option<EconomicsReceipt> = if economics_path.exists() {
        match fs::read_to_string(&economics_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Extract metadata for swarm-meta block
    let run_id = gate_receipt
        .as_ref()
        .map(|r| r.run_id.clone())
        .unwrap_or_else(|| format!("unknown-pr{}", args.pr));

    let commit =
        gate_receipt.as_ref().map(|r| r.commit.clone()).unwrap_or_else(|| "unknown".to_string());

    // Generate cover sheet markdown
    let mut content = String::new();

    // Idempotent start marker
    content.push_str(COVER_SHEET_START);
    content.push('\n');
    content.push_str("## Cover Sheet\n\n");

    // What changed section
    content.push_str("### What changed\n");
    let description =
        args.description.unwrap_or_else(|| "<TODO: 1-3 sentences describing the change>".into());
    content.push_str(&format!("- {}\n\n", description));

    // Review map section
    content.push_str("### Where to look (review map)\n");
    content.push_str("| Area | Files | Why |\n");
    content.push_str("|------|-------|-----|\n");
    content.push_str("| <domain> | `path/to/files` | <what changed here> |\n\n");

    // Proof/receipts section
    content.push_str("### Proof (receipts)\n");
    content.push_str("| Check | Status | Receipt |\n");
    content.push_str("|-------|--------|--------|\n");

    // Gate receipt status
    if let Some(ref gate) = gate_receipt {
        let overall_status = format_status(gate.overall_status);
        content.push_str(&format!(
            "| Policy (gate) | {} | `{}` |\n",
            overall_status,
            gate_path.display()
        ));

        // Add individual gate results
        for gate_result in &gate.gates {
            let status = format_status(gate_result.status);
            content.push_str(&format!("| - {} | {} | |\n", gate_result.name, status));
        }
    } else if gate_path.exists() {
        content.push_str(&format!(
            "| Policy (gate) | ? | `{}` (parse error) |\n",
            gate_path.display()
        ));
    } else {
        content.push_str("| Policy (gate) | N/A | (no receipt found) |\n");
    }

    content.push('\n');

    // Errata section (proper format)
    content.push_str("### Errata (what we got wrong)\n");
    content.push_str("- Nothing identified in this PR's scope.\n");
    content.push_str(
        "- (If you find something later, add an addendum here and link the fixing PR.)\n\n",
    );

    // Unified budget section
    content.push_str("### Unified budget (DevLT dominates)\n");
    content.push_str("| Metric | Value | Notes |\n");
    content.push_str("|--------|-------|-------|\n");

    let (author_devlt, compute_usd) = if let Some(ref econ) = economics {
        let author = econ
            .devlt_minutes
            .author
            .map(|m| format!("~{} min", m))
            .unwrap_or_else(|| "unknown".to_string());

        let compute = econ
            .compute
            .tokens_usd
            .map(|c| format!("~${:.2}", c))
            .unwrap_or_else(|| "unknown".to_string());

        (author, compute)
    } else if economics_path.exists() {
        ("? (parse error)".to_string(), "? (parse error)".to_string())
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };

    content.push_str(&format!("| DevLT (author) | {} | |\n", author_devlt));
    content.push_str("| DevLT (review) | unknown | |\n");
    content.push_str(&format!("| Compute spend | {} | |\n", compute_usd));

    content.push('\n');

    // Reproduce locally section
    content.push_str("### Reproduce locally\n");
    content.push_str("```bash\n");
    content.push_str("nix develop\n");
    content.push_str("cargo xtask selftest\n");
    content.push_str("```\n\n");

    // Swarm-meta block (machine-updated, idempotent)
    let swarm_meta = SwarmMeta {
        run_id: run_id.clone(),
        pr: args.pr,
        commit,
        receipts: ReceiptPaths {
            gate: if gate_path.exists() { Some(gate_path.display().to_string()) } else { None },
            economics: if economics_path.exists() {
                Some(economics_path.display().to_string())
            } else {
                None
            },
        },
        devlt_minutes: DevLtMeta { author: Some(author_devlt.clone()), review: None },
        compute: ComputeMeta { tokens_usd: Some(compute_usd.clone()) },
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    content.push_str("<!-- swarm-meta (machine-updated; do not hand edit)\n");
    content.push_str(&serde_yaml::to_string(&swarm_meta).unwrap_or_default());
    content.push_str("-->\n");

    // Idempotent end marker
    content.push_str(COVER_SHEET_END);
    content.push('\n');

    // Output
    match args.output {
        Some(path) => {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(&path, &content)
                .with_context(|| format!("Failed to write to {}", path.display()))?;
            println!("{} Cover sheet written to {}", "OK".green(), path.display());
        }
        None => {
            println!("\n{}", content);
        }
    }

    Ok(())
}

/// Format gate status for display
fn format_status(status: GateStatus) -> &'static str {
    match status {
        GateStatus::Pass => "PASS",
        GateStatus::Fail => "FAIL",
        GateStatus::Skipped => "N/A",
    }
}

/// Extract cover sheet from markdown using idempotent markers
#[allow(dead_code)] // Used by pr_update for reading errata, and in tests
pub fn extract_cover_sheet(markdown: &str) -> Option<&str> {
    let start_idx = markdown.find(COVER_SHEET_START)?;
    let end_idx = markdown.find(COVER_SHEET_END)?;
    if end_idx > start_idx {
        Some(&markdown[start_idx..end_idx + COVER_SHEET_END.len()])
    } else {
        None
    }
}

/// Replace cover sheet in markdown, preserving content outside markers
pub fn replace_cover_sheet(markdown: &str, new_cover_sheet: &str) -> String {
    if let Some(start_idx) = markdown.find(COVER_SHEET_START)
        && let Some(end_idx) = markdown.find(COVER_SHEET_END)
    {
        let end_pos = end_idx + COVER_SHEET_END.len();
        let mut result = String::new();
        result.push_str(&markdown[..start_idx]);
        result.push_str(new_cover_sheet);
        result.push_str(&markdown[end_pos..]);
        return result;
    }
    // No existing cover sheet, prepend it
    format!("{}\n\n{}", new_cover_sheet, markdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_run_dir() {
        let args = PrCoverArgs { pr: 123, run_dir: None, output: None, description: None };

        let expected = PathBuf::from(".runs/pr/123/latest");
        let actual =
            args.run_dir.unwrap_or_else(|| PathBuf::from(format!(".runs/pr/{}/latest", args.pr)));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_custom_run_dir() {
        let args = PrCoverArgs {
            pr: 456,
            run_dir: Some(PathBuf::from("/custom/path")),
            output: None,
            description: None,
        };

        assert_eq!(args.run_dir, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_extract_cover_sheet() {
        let markdown = r#"Some intro text

<!-- pr-cover-sheet:start -->
## Cover Sheet

### What changed
- Did stuff
<!-- pr-cover-sheet:end -->

Some trailing text
"#;

        let extracted = extract_cover_sheet(markdown).unwrap();
        assert!(extracted.starts_with("<!-- pr-cover-sheet:start -->"));
        assert!(extracted.ends_with("<!-- pr-cover-sheet:end -->"));
        assert!(extracted.contains("## Cover Sheet"));
    }

    #[test]
    fn test_extract_cover_sheet_not_found() {
        let markdown = "Just some regular markdown without markers";
        assert!(extract_cover_sheet(markdown).is_none());
    }

    #[test]
    fn test_replace_cover_sheet_existing() {
        let original = r#"Intro

<!-- pr-cover-sheet:start -->
OLD CONTENT
<!-- pr-cover-sheet:end -->

Footer"#;

        let new_sheet = "<!-- pr-cover-sheet:start -->\nNEW CONTENT\n<!-- pr-cover-sheet:end -->";
        let result = replace_cover_sheet(original, new_sheet);

        assert!(result.contains("Intro"));
        assert!(result.contains("NEW CONTENT"));
        assert!(result.contains("Footer"));
        assert!(!result.contains("OLD CONTENT"));
    }

    #[test]
    fn test_replace_cover_sheet_none_existing() {
        let original = "Just some content";
        let new_sheet = "<!-- pr-cover-sheet:start -->\nCOVER\n<!-- pr-cover-sheet:end -->";
        let result = replace_cover_sheet(original, new_sheet);

        assert!(result.starts_with("<!-- pr-cover-sheet:start -->"));
        assert!(result.contains("Just some content"));
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(GateStatus::Pass), "PASS");
        assert_eq!(format_status(GateStatus::Fail), "FAIL");
        assert_eq!(format_status(GateStatus::Skipped), "N/A");
    }

    #[test]
    fn test_idempotent_markers_present() {
        // Verify the constants are properly defined
        assert!(COVER_SHEET_START.starts_with("<!--"));
        assert!(COVER_SHEET_END.starts_with("<!--"));
        assert!(COVER_SHEET_START.contains("start"));
        assert!(COVER_SHEET_END.contains("end"));
    }
}
