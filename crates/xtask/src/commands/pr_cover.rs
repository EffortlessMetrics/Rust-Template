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
use gov_receipts::{
    BoundaryRating, EconomicsReceipt, GateReceipt, GateStatus, ProbeStatus, QualityReceipt,
    TelemetryReceipt, TimelineConfidence, TimelineReceipt, Topology,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    quality: Option<String>,
    telemetry: Option<String>,
    timeline: Option<String>,
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
    let quality_path = run_dir.join("receipts/quality.json");
    let telemetry_path = run_dir.join("receipts/telemetry.json");
    let timeline_path = run_dir.join("receipts/timeline.json");

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

    // Try to load quality receipt
    let quality: Option<QualityReceipt> = if quality_path.exists() {
        match fs::read_to_string(&quality_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Try to load telemetry receipt
    let telemetry: Option<TelemetryReceipt> = if telemetry_path.exists() {
        match fs::read_to_string(&telemetry_path) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Try to load timeline receipt
    let timeline: Option<TimelineReceipt> = if timeline_path.exists() {
        match fs::read_to_string(&timeline_path) {
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

    // Quality summary section
    content.push_str(&format_quality_section(&quality, &quality_path));

    // Telemetry summary section
    content.push_str(&format_telemetry_section(&telemetry, &telemetry_path));

    // Timeline/friction section
    content.push_str(&format_timeline_section(&timeline, &timeline_path));

    // Review path section - dynamic guidance based on signals
    content.push_str(&format_review_path_section(&quality, &telemetry, &timeline));

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
            quality: if quality_path.exists() {
                Some(quality_path.display().to_string())
            } else {
                None
            },
            telemetry: if telemetry_path.exists() {
                Some(telemetry_path.display().to_string())
            } else {
                None
            },
            timeline: if timeline_path.exists() {
                Some(timeline_path.display().to_string())
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

/// Format quality summary section from quality receipt.
///
/// Returns empty string if receipt is None (section is skipped).
fn format_quality_section(quality: &Option<QualityReceipt>, path: &Path) -> String {
    let Some(q) = quality else {
        return String::new();
    };

    let mut s = String::new();
    s.push_str("### Quality summary\n\n");

    // Contract changes subsection
    s.push_str("**Contract changes:**\n");
    let contract = &q.quality.contract;
    let has_contract_changes = contract.public_api.as_ref().is_some_and(|c| c.changed)
        || contract.schema.as_ref().is_some_and(|c| c.changed)
        || contract.cli.as_ref().is_some_and(|c| c.changed);

    if has_contract_changes {
        let mut changes = Vec::new();
        if let Some(ref api) = contract.public_api
            && api.changed
        {
            let breaking = if api.breaking { " (BREAKING)" } else { "" };
            changes.push(format!("Public API{}", breaking));
        }
        if let Some(ref schema) = contract.schema
            && schema.changed
        {
            let breaking = if schema.breaking { " (BREAKING)" } else { "" };
            changes.push(format!("Schema{}", breaking));
        }
        if let Some(ref cli) = contract.cli
            && cli.changed
        {
            let breaking = if cli.breaking { " (BREAKING)" } else { "" };
            changes.push(format!("CLI{}", breaking));
        }
        s.push_str(&format!("- {}\n", changes.join(", ")));
    } else {
        s.push_str("- No contract surface changes\n");
    }

    // Boundary integrity subsection
    s.push_str("\n**Boundary integrity:**\n");
    let boundaries = &q.quality.boundaries;
    s.push_str(&format!("- Modules touched: {}\n", boundaries.modules_touched));
    if !boundaries.hotspots.is_empty() {
        s.push_str(&format!("- Hotspots: {}\n", boundaries.hotspots.join(", ")));
    }
    if let Some(ref assessment) = boundaries.llm_assessment {
        let rating = match assessment.rating {
            BoundaryRating::Improved => "Improved",
            BoundaryRating::Neutral => "Neutral",
            BoundaryRating::Degraded => "Degraded",
        };
        s.push_str(&format!("- Boundary rating: {}\n", rating));
    }

    // Verification metrics subsection
    s.push_str("\n**Verification metrics:**\n");
    let verification = &q.quality.verification;
    s.push_str(&format!("- Tests added: {} LOC\n", verification.tests_added_loc));
    s.push_str(&format!("- Impl added: {} LOC\n", verification.impl_added_loc));
    if verification.test_density_delta != 0.0 {
        let sign = if verification.test_density_delta > 0.0 { "+" } else { "" };
        s.push_str(&format!(
            "- Test density delta: {}{:.2}\n",
            sign, verification.test_density_delta
        ));
    }

    // Risk indicators subsection
    s.push_str("\n**Risk indicators:**\n");
    let risks = &q.quality.risks;
    let mut has_risks = false;
    if let Some(ref unsafe_delta) = risks.unsafe_delta
        && (unsafe_delta.added > 0 || unsafe_delta.removed > 0)
    {
        // Quality receipt uses keyword scan from git diff (heuristic)
        s.push_str(&format!(
            "- Unsafe keyword delta: +{} / -{} _(heuristic)_\n",
            unsafe_delta.added, unsafe_delta.removed
        ));
        has_risks = true;
    }
    if !risks.deps_added.is_empty() {
        s.push_str(&format!("- Dependencies added: {}\n", risks.deps_added.join(", ")));
        has_risks = true;
    }
    if !risks.concurrency_primitives_added.is_empty() {
        s.push_str(&format!(
            "- Concurrency primitives: {}\n",
            risks.concurrency_primitives_added.join(", ")
        ));
        has_risks = true;
    }
    if !has_risks {
        s.push_str("- No elevated risk indicators\n");
    }

    s.push_str(&format!("\n`Receipt: {}`\n\n", path.display()));
    s
}

/// Format telemetry summary section from telemetry receipt.
///
/// Returns empty string if receipt is None (section is skipped).
fn format_telemetry_section(telemetry: &Option<TelemetryReceipt>, path: &Path) -> String {
    let Some(t) = telemetry else {
        return String::new();
    };

    let mut s = String::new();
    s.push_str("### Telemetry summary\n\n");

    // Change surface stats
    s.push_str("**Change surface:**\n");
    let surface = &t.change_surface;
    s.push_str(&format!(
        "- {} files | +{} / -{} lines\n",
        surface.files_changed, surface.insertions, surface.deletions
    ));
    if !surface.crates_touched.is_empty() {
        s.push_str(&format!("- Crates touched: {}\n", surface.crates_touched.join(", ")));
    }

    // Probe execution summary
    s.push_str("\n**Probe execution:**\n");
    let executed = t.probes.iter().filter(|p| p.status == ProbeStatus::Run).count();
    let errored = t.probes.iter().filter(|p| p.status == ProbeStatus::Error).count();
    let skipped = t.not_run.len();
    s.push_str(&format!("- Executed: {} | Skipped: {} | Errors: {}\n", executed, skipped, errored));

    if errored > 0 {
        let error_probes: Vec<_> =
            t.probes.iter().filter(|p| p.status == ProbeStatus::Error).map(|p| &p.name).collect();
        s.push_str(&format!(
            "- Failed probes: {}\n",
            error_probes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    // Contract drift detection
    if let Some(ref contracts) = t.contracts
        && (contracts.schema_changed || contracts.public_api_changed || contracts.cli_changed)
    {
        s.push_str("\n**Contract drift detected:**\n");
        if contracts.schema_changed {
            s.push_str("- Schema changed\n");
        }
        if contracts.public_api_changed {
            s.push_str("- Public API changed\n");
        }
        if contracts.cli_changed {
            s.push_str("- CLI changed\n");
        }
        if contracts.breaking {
            s.push_str("- **BREAKING changes detected**\n");
        }
    }

    // Safety/unsafe code metrics - distinguish heuristic vs measured
    if let Some(ref safety) = t.safety
        && (safety.unsafe_added > 0 || safety.unsafe_removed > 0 || safety.geiger_summary.is_some())
    {
        s.push_str("\n**Safety analysis:**\n");
        // Distinguish between measured (cargo-geiger) and heuristic (git diff keyword scan)
        if safety.geiger_summary.is_some() {
            s.push_str(&format!(
                "- Unsafe delta: +{} / -{} _(measured by cargo-geiger)_\n",
                safety.unsafe_added, safety.unsafe_removed
            ));
            if let Some(ref geiger) = safety.geiger_summary {
                s.push_str(&format!(
                    "- Total unsafe: {} used, {} unused",
                    geiger.used_unsafe, geiger.unused_unsafe
                ));
                if geiger.forbid_unsafe {
                    s.push_str(" (forbid(unsafe_code) set)");
                }
                s.push('\n');
            }
        } else if safety.unsafe_added > 0 || safety.unsafe_removed > 0 {
            s.push_str(&format!(
                "- Unsafe keyword delta: +{} / -{} _(heuristic from git diff)_\n",
                safety.unsafe_added, safety.unsafe_removed
            ));
        }
    }

    s.push_str(&format!("\n`Receipt: {}`\n\n", path.display()));
    s
}

/// Format timeline/friction section from timeline receipt.
///
/// Returns empty string if receipt is None (section is skipped).
fn format_timeline_section(timeline: &Option<TimelineReceipt>, path: &Path) -> String {
    let Some(t) = timeline else {
        return String::new();
    };

    let mut s = String::new();
    s.push_str("### Timeline & friction log\n\n");

    // Duration and sessions
    s.push_str("**Duration:**\n");
    if let Some(minutes) = t.wall_clock.total_duration_minutes {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if hours > 0 {
            s.push_str(&format!("- Total: {}h {}m\n", hours, mins));
        } else {
            s.push_str(&format!("- Total: {}m\n", mins));
        }
    }
    s.push_str(&format!("- Sessions: {}\n", t.sessions.len()));
    s.push_str(&format!("- Total commits: {}\n", t.total_commits()));

    // Friction zones
    if !t.friction_zones.is_empty() {
        s.push_str("\n**Friction zones** (files touched repeatedly):\n");
        for zone in &t.friction_zones {
            s.push_str(&format!("- `{}` ({} touches)\n", zone.path, zone.touch_count));
        }
    }

    // Oscillations (design uncertainty)
    if !t.oscillations.is_empty() {
        s.push_str("\n**Oscillations** (design uncertainty signals):\n");
        for osc in &t.oscillations {
            s.push_str(&format!(
                "- {}: `{}`\n",
                format_oscillation_type(osc.oscillation_type),
                osc.subject
            ));
        }
    }

    // Topology classification
    s.push_str("\n**Topology:**\n");
    let (topo_str, topo_emoji) = match t.topology {
        Topology::Linear => ("Linear", ""),
        Topology::Cyclical => ("Cyclical", " (iterative refinement)"),
        Topology::Chaotic => ("Chaotic", " **[REVIEW SIGNAL]**"),
    };
    s.push_str(&format!("- Classification: {}{}\n", topo_str, topo_emoji));

    if let Some(conf) = t.topology_confidence {
        let conf_str = match conf {
            TimelineConfidence::High => "High",
            TimelineConfidence::Medium => "Medium",
            TimelineConfidence::Low => "Low",
        };
        s.push_str(&format!("- Confidence: {}\n", conf_str));
    }

    // Highlight chaotic/cyclical as signals
    if matches!(t.topology, Topology::Chaotic | Topology::Cyclical) && t.is_high_friction() {
        s.push_str("\n> **Note:** This PR shows signs of exploration/iteration. ");
        s.push_str("Consider whether the final design is stable.\n");
    }

    s.push_str(&format!("\n`Receipt: {}`\n\n", path.display()));
    s
}

// ============================================================================
// Review Path Thresholds (prevent noise in small PRs)
// ============================================================================

/// Minimum friction zones before signaling (applies to total)
const MIN_FRICTION_ZONES_TOTAL: usize = 3;

/// Minimum friction zones in core_code to always signal (more sensitive for core code)
const MIN_CORE_CODE_FRICTION: usize = 1;

/// Only emit "no tests added" if impl exceeds this LOC
const MIN_IMPL_FOR_TEST_WARNING: u32 = 100;

// ============================================================================
// Review Path Guidance Strings (deduplicated)
// ============================================================================

const GUIDANCE_REVIEW_HOTSPOTS: &str = "Review hotspots first - these files had the most churn";
const GUIDANCE_SCAN_FRICTION: &str = "Scan friction zones for potential design issues";
const GUIDANCE_CHECK_OSCILLATIONS: &str = "Check oscillation subjects for dependency/design churn";
const GUIDANCE_UNSAFE_JUSTIFICATION: &str = "Require justification for unsafe code additions";
const GUIDANCE_VERIFY_TEST_COVERAGE: &str = "Verify failure modes are covered by existing tests";
const GUIDANCE_VERIFY_MIGRATION: &str = "Verify migration path for downstream consumers";
const GUIDANCE_VERIFY_SCHEMA: &str = "Verify schema migrations are backwards-compatible";

/// Categorize a file path for friction zone analysis.
/// Used to determine if friction zones are in "core_code" vs other categories.
///
/// Test paths are checked first to ensure test_fixtures/ under crates/ are
/// correctly classified as tests, not core_code.
fn categorize_path_for_review(path: &str) -> &'static str {
    // Normalize Windows separators for consistent matching
    let p = path.replace('\\', "/");

    // Test paths - check FIRST to catch test_fixtures/, etc. under crates/
    if p.contains("/tests/")
        || p.contains("/test_fixtures/")
        || p.contains("_test.rs")
        || p.ends_with("_tests.rs")
        || p.contains("/benches/")
    {
        return "tests";
    }

    // Documentation
    if p.starts_with("docs/") || p.ends_with(".md") {
        return "docs";
    }

    // Core code paths
    if p.starts_with("src/")
        || p.starts_with("crates/")
        || p.starts_with("lib/")
        || p.starts_with("core/")
        || p.starts_with("packages/")
        || p.starts_with("apps/")
    {
        return "core_code";
    }

    "other"
}

/// Format review path section based on signals from all receipts.
/// Provides specific guidance to reviewers based on detected patterns.
/// Uses thresholds to avoid noise in small PRs.
fn format_review_path_section(
    quality: &Option<QualityReceipt>,
    telemetry: &Option<TelemetryReceipt>,
    timeline: &Option<TimelineReceipt>,
) -> String {
    let mut signals: Vec<String> = Vec::new();
    let mut guidance: Vec<String> = Vec::new();

    // Check timeline signals with thresholds
    if let Some(t) = timeline {
        // Chaotic topology is always significant
        if matches!(t.topology, Topology::Chaotic) {
            signals.push("Chaotic topology detected".to_string());
            guidance.push(GUIDANCE_REVIEW_HOTSPOTS.to_string());
        }

        // Apply thresholds to friction zones
        // Only signal if: zones > MIN_FRICTION_ZONES_TOTAL OR core_code zones > MIN_CORE_CODE_FRICTION
        if !t.friction_zones.is_empty() {
            let core_code_zones = t
                .friction_zones
                .iter()
                .filter(|z| categorize_path_for_review(&z.path) == "core_code")
                .count();

            let should_signal_friction = t.friction_zones.len() >= MIN_FRICTION_ZONES_TOTAL
                || core_code_zones >= MIN_CORE_CODE_FRICTION;

            if should_signal_friction {
                signals.push(format!(
                    "{} friction zones ({} in core)",
                    t.friction_zones.len(),
                    core_code_zones
                ));
                guidance.push(GUIDANCE_SCAN_FRICTION.to_string());
            }
        }

        // Filter oscillations: only signal if involves deps or core_code (not docs churn)
        if !t.oscillations.is_empty() {
            let significant_oscillations: Vec<_> = t
                .oscillations
                .iter()
                .filter(|osc| {
                    matches!(osc.oscillation_type, gov_receipts::OscillationType::Dependency)
                        || categorize_path_for_review(&osc.subject) == "core_code"
                })
                .collect();

            if !significant_oscillations.is_empty() {
                signals
                    .push(format!("{} significant oscillation(s)", significant_oscillations.len()));
                guidance.push(GUIDANCE_CHECK_OSCILLATIONS.to_string());
            }
        }
    }

    // Check quality signals
    if let Some(q) = quality {
        // Unsafe delta check - always signal unsafe additions
        if let Some(ref ud) = q.quality.risks.unsafe_delta
            && ud.added > 0
        {
            signals.push(format!("+{} unsafe", ud.added));
            guidance.push(GUIDANCE_UNSAFE_JUSTIFICATION.to_string());
        }

        // Test coverage check with threshold
        let tests = q.quality.verification.tests_added_loc;
        let impl_loc = q.quality.verification.impl_added_loc;
        if impl_loc > MIN_IMPL_FOR_TEST_WARNING && tests == 0 {
            signals.push("No tests added".to_string());
            guidance.push(GUIDANCE_VERIFY_TEST_COVERAGE.to_string());
        }

        // Contract changes
        if let Some(ref api) = q.quality.contract.public_api
            && api.breaking
        {
            signals.push("Breaking API change".to_string());
            guidance.push(GUIDANCE_VERIFY_MIGRATION.to_string());
        }
    }

    // Check telemetry signals
    if let Some(t) = telemetry
        && let Some(ref contracts) = t.contracts
        && contracts.schema_changed
    {
        signals.push("Schema changed".to_string());
        guidance.push(GUIDANCE_VERIFY_SCHEMA.to_string());
    }

    // Don't show section if no signals
    if signals.is_empty() {
        return String::new();
    }

    // Deduplicate guidance (same guidance can be triggered by multiple signals)
    // Use stable-order dedup (HashSet) instead of adjacent-only dedup
    {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        guidance.retain(|g| seen.insert(g.clone()));
    }

    let mut s = String::new();
    s.push_str("### Review path (suggested)\n\n");

    // Signal summary
    s.push_str("**Signals detected:**\n");
    for signal in &signals {
        s.push_str(&format!("- {}\n", signal));
    }

    // Guidance list
    if !guidance.is_empty() {
        s.push_str("\n**Recommended review actions:**\n");
        for (i, g) in guidance.iter().enumerate() {
            s.push_str(&format!("{}. {}\n", i + 1, g));
        }
    }

    s.push('\n');
    s
}

/// Format oscillation type for display.
fn format_oscillation_type(ot: gov_receipts::OscillationType) -> &'static str {
    match ot {
        gov_receipts::OscillationType::Dependency => "Dependency",
        gov_receipts::OscillationType::File => "File",
        gov_receipts::OscillationType::Feature => "Feature",
        gov_receipts::OscillationType::Approach => "Approach",
    }
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

    #[test]
    fn test_format_quality_section_none() {
        let path = PathBuf::from("receipts/quality.json");
        let result = format_quality_section(&None, &path);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_quality_section_with_data() {
        use gov_receipts::{
            Boundaries, Contract, ContractChange, Quality, Risks, UnsafeDelta, Verification,
        };

        let receipt = QualityReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: Some("test".to_string()),
            quality: Quality {
                contract: Contract {
                    public_api: Some(ContractChange {
                        changed: true,
                        breaking: true,
                        evidence: vec![],
                    }),
                    ..Default::default()
                },
                boundaries: Boundaries {
                    modules_touched: 5,
                    hotspots: vec!["lib.rs".to_string()],
                    ..Default::default()
                },
                verification: Verification {
                    tests_added_loc: 100,
                    impl_added_loc: 200,
                    test_density_delta: 0.15,
                    ..Default::default()
                },
                risks: Risks {
                    unsafe_delta: Some(UnsafeDelta { added: 1, removed: 0 }),
                    deps_added: vec!["serde".to_string()],
                    ..Default::default()
                },
            },
            meta: None,
        };

        let path = PathBuf::from("receipts/quality.json");
        let result = format_quality_section(&Some(receipt), &path);

        assert!(result.contains("### Quality summary"));
        assert!(result.contains("Public API (BREAKING)"));
        assert!(result.contains("Modules touched: 5"));
        assert!(result.contains("Hotspots: lib.rs"));
        assert!(result.contains("Tests added: 100 LOC"));
        assert!(result.contains("Test density delta: +0.15"));
        assert!(result.contains("Unsafe keyword delta: +1 / -0 _(heuristic)_"));
        assert!(result.contains("Dependencies added: serde"));
    }

    #[test]
    fn test_format_telemetry_section_none() {
        let path = PathBuf::from("receipts/telemetry.json");
        let result = format_telemetry_section(&None, &path);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_telemetry_section_with_data() {
        use gov_receipts::{ChangeSurface, Contracts, ProbeResult};

        let receipt = TelemetryReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: "test".to_string(),
            profile: None,
            change_surface: ChangeSurface {
                files_changed: 10,
                insertions: 500,
                deletions: 200,
                crates_touched: vec!["xtask".to_string(), "gov-receipts".to_string()],
                ..Default::default()
            },
            contracts: Some(Contracts {
                schema_changed: true,
                breaking: true,
                ..Default::default()
            }),
            safety: None,
            structure: None,
            verification: None,
            probes: vec![
                ProbeResult {
                    name: "clippy".to_string(),
                    version: None,
                    status: ProbeStatus::Run,
                    reason: None,
                    duration_ms: None,
                    artifact_path: None,
                },
                ProbeResult {
                    name: "test".to_string(),
                    version: None,
                    status: ProbeStatus::Error,
                    reason: Some("timeout".to_string()),
                    duration_ms: None,
                    artifact_path: None,
                },
            ],
            not_run: vec![],
            meta: None,
        };

        let path = PathBuf::from("receipts/telemetry.json");
        let result = format_telemetry_section(&Some(receipt), &path);

        assert!(result.contains("### Telemetry summary"));
        assert!(result.contains("10 files | +500 / -200 lines"));
        assert!(result.contains("Crates touched: xtask, gov-receipts"));
        assert!(result.contains("Executed: 1 | Skipped: 0 | Errors: 1"));
        assert!(result.contains("Failed probes: test"));
        assert!(result.contains("Schema changed"));
        assert!(result.contains("BREAKING changes detected"));
    }

    #[test]
    fn test_format_timeline_section_none() {
        let path = PathBuf::from("receipts/timeline.json");
        let result = format_timeline_section(&None, &path);
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_timeline_section_with_data() {
        use gov_receipts::{FrictionZone, Oscillation, OscillationType, Session, WallClock};

        let receipt = TimelineReceipt {
            schema_version: "1.0".to_string(),
            pr: Some(123),
            run_id: "test".to_string(),
            wall_clock: WallClock {
                first_commit: "2026-01-07T10:00:00Z".parse().unwrap(),
                last_commit: "2026-01-07T14:00:00Z".parse().unwrap(),
                pr_created: None,
                pr_merged: None,
                total_duration_minutes: Some(150),
            },
            sessions: vec![Session {
                start: "2026-01-07T10:00:00Z".parse().unwrap(),
                end: "2026-01-07T12:00:00Z".parse().unwrap(),
                commit_count: 5,
                classification: None,
            }],
            friction_zones: vec![FrictionZone {
                path: "lib.rs".to_string(),
                touch_count: 8,
                commits: vec![],
            }],
            oscillations: vec![Oscillation {
                oscillation_type: OscillationType::Dependency,
                subject: "serde".to_string(),
                sequence: vec![],
            }],
            convergence: None,
            topology: Topology::Chaotic,
            topology_confidence: Some(TimelineConfidence::High),
            topology_reasons: vec![],
            events: vec![],
            meta: None,
        };

        let path = PathBuf::from("receipts/timeline.json");
        let result = format_timeline_section(&Some(receipt), &path);

        assert!(result.contains("### Timeline & friction log"));
        assert!(result.contains("Total: 2h 30m"));
        assert!(result.contains("Sessions: 1"));
        assert!(result.contains("Total commits: 5"));
        assert!(result.contains("`lib.rs` (8 touches)"));
        assert!(result.contains("Dependency: `serde`"));
        assert!(result.contains("Classification: Chaotic **[REVIEW SIGNAL]**"));
        assert!(result.contains("Confidence: High"));
        assert!(result.contains("This PR shows signs of exploration/iteration"));
    }

    #[test]
    fn test_categorize_path_for_review() {
        // Core code paths
        assert_eq!(categorize_path_for_review("src/lib.rs"), "core_code");
        assert_eq!(categorize_path_for_review("crates/foo/src/lib.rs"), "core_code");

        // Test paths - including test_fixtures under crates/
        assert_eq!(categorize_path_for_review("crates/foo/test_fixtures/x.json"), "tests");
        assert_eq!(categorize_path_for_review("crates/foo/tests/integration.rs"), "tests");
        assert_eq!(categorize_path_for_review("src/foo_test.rs"), "tests");
        assert_eq!(categorize_path_for_review("src/foo_tests.rs"), "tests");
        assert_eq!(categorize_path_for_review("crates/foo/benches/bench.rs"), "tests");

        // Documentation
        assert_eq!(categorize_path_for_review("docs/README.md"), "docs");
        assert_eq!(categorize_path_for_review("CHANGELOG.md"), "docs");

        // Other
        assert_eq!(categorize_path_for_review("Cargo.toml"), "other");
        assert_eq!(categorize_path_for_review(".github/workflows/ci.yml"), "other");

        // Windows path normalization
        assert_eq!(categorize_path_for_review("crates\\foo\\test_fixtures\\x.json"), "tests");
    }
}
