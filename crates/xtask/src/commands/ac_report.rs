//! AC Report: Downstream consumer of `ac-status --json`
//!
//! This command provides human-readable views of AC governance state,
//! treating `ac-status --json` as its data source.
//!
//! # Design
//!
//! See `docs/design/DESIGN-AC-REPORT-CONSUMER.md` for the full design.
//!
//! # Usage
//!
//! ```bash
//! # Default: human-readable summary
//! cargo xtask ac-report
//!
//! # Kernel-only view (must_have_ac=true)
//! cargo xtask ac-report --must-have
//!
//! # Filter by status
//! cargo xtask ac-report --status unknown
//!
//! # Markdown output for PRs
//! cargo xtask ac-report --format markdown
//! ```

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Write;

/// Arguments for ac-report command
#[derive(Debug, Clone)]
pub struct AcReportArgs {
    /// Only show must_have_ac=true ACs
    pub must_have: bool,
    /// Filter by status (pass, fail, unknown)
    pub status: Option<String>,
    /// Group by story instead of requirement
    pub by_story: bool,
    /// Output format (text, markdown, html, json)
    pub format: String,
}

impl Default for AcReportArgs {
    fn default() -> Self {
        Self { must_have: false, status: None, by_story: false, format: "text".to_string() }
    }
}

// ===========================================================================
// Data types - mirror ac_status.rs JSON output
// ===========================================================================

/// Deserialized AC status report (from ac-status --json)
#[derive(Debug, Deserialize)]
pub struct AcReport {
    pub schema_version: String,
    pub timestamp: String,
    pub must_have_acs: AcCategoryStats,
    pub optional_acs: AcCategoryStats,
    pub coverage_percent: f64,
    pub acs: Vec<AcJson>,
}

#[derive(Debug, Deserialize)]
pub struct AcCategoryStats {
    pub total: usize,
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
}

#[derive(Debug, Deserialize)]
pub struct AcJson {
    pub id: String,
    pub story_id: String,
    pub req_id: String,
    pub text: String,
    pub status: String,
    pub source: String,
    pub must_have_ac: bool,
    #[serde(default)]
    pub scenarios: Vec<String>,
    pub tests_total: usize,
    pub tests_executed: usize,
}

impl AcReport {
    /// Load by calling ac-status --json internally
    pub fn load() -> Result<Self> {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(["xtask", "ac-status", "--json"])
            .env("XTASK_NO_REGEN", "1") // Avoid regenerating BDD if missing
            .output()
            .context("Failed to run ac-status --json")?;

        if !output.status.success() {
            // ac-status returns non-zero if ACs are failing, but still outputs JSON
            // Only fail if we got no output
            if output.stdout.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("ac-status --json produced no output: {}", stderr);
            }
        }

        let json = String::from_utf8_lossy(&output.stdout);
        Self::from_json(&json)
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse ac-status JSON")
    }

    /// Filter ACs by criteria
    pub fn filter<'a>(&'a self, args: &AcReportArgs) -> Vec<&'a AcJson> {
        self.acs
            .iter()
            .filter(|ac| {
                // Filter by must_have_ac
                if args.must_have && !ac.must_have_ac {
                    return false;
                }
                // Filter by status
                if let Some(ref status) = args.status
                    && ac.status != *status
                {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Group ACs by requirement
    pub fn by_requirement<'a>(&'a self, acs: &[&'a AcJson]) -> BTreeMap<String, Vec<&'a AcJson>> {
        let mut groups: BTreeMap<String, Vec<&AcJson>> = BTreeMap::new();
        for ac in acs {
            groups.entry(ac.req_id.clone()).or_default().push(ac);
        }
        groups
    }

    /// Group ACs by story
    pub fn by_story<'a>(&'a self, acs: &[&'a AcJson]) -> BTreeMap<String, Vec<&'a AcJson>> {
        let mut groups: BTreeMap<String, Vec<&AcJson>> = BTreeMap::new();
        for ac in acs {
            groups.entry(ac.story_id.clone()).or_default().push(ac);
        }
        groups
    }
}

// ===========================================================================
// Main entry point
// ===========================================================================

pub fn run(args: AcReportArgs) -> Result<()> {
    // Load the report
    let report = AcReport::load()?;

    // Check schema version
    if report.schema_version != "2.0" {
        eprintln!(
            "{} Unknown schema version '{}', expected '2.0'. Output may be incorrect.",
            "[WARN]".yellow(),
            report.schema_version
        );
    }

    // Dispatch to format handler
    match args.format.as_str() {
        "text" => render_text(&report, &args),
        "markdown" => render_markdown(&report, &args),
        "json" => render_json(&report),
        "html" => render_html(&report, &args),
        _ => anyhow::bail!("Unknown format: {}. Use text, markdown, html, or json.", args.format),
    }
}

// ===========================================================================
// Text output (default)
// ===========================================================================

fn render_text(report: &AcReport, args: &AcReportArgs) -> Result<()> {
    println!("{}", "AC Governance Report".cyan().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Summary
    println!("{}", "Summary:".bold());
    println!(
        "  Must-have ACs: {} total ({} passing, {} failing, {} unknown)",
        report.must_have_acs.total,
        report.must_have_acs.passing.to_string().green(),
        report.must_have_acs.failing.to_string().red(),
        report.must_have_acs.unknown.to_string().yellow()
    );
    println!(
        "  Optional ACs:  {} total ({} passing, {} failing, {} unknown)",
        report.optional_acs.total,
        report.optional_acs.passing.to_string().green(),
        report.optional_acs.failing.to_string().red(),
        report.optional_acs.unknown.to_string().yellow()
    );
    println!("  Coverage:      {:.1}%", report.coverage_percent);
    println!();

    // Filter ACs
    let filtered = report.filter(args);

    if filtered.is_empty() {
        let filter_desc = describe_filter(args);
        println!("{} No ACs match filter: {}", "✓".green(), filter_desc);
        return Ok(());
    }

    // Group and display
    let groups =
        if args.by_story { report.by_story(&filtered) } else { report.by_requirement(&filtered) };

    let group_label = if args.by_story { "Story" } else { "Requirement" };
    let filter_desc = describe_filter(args);
    println!("{} Filtered by: {}", "📋".bold(), filter_desc);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    for (group_id, acs) in groups {
        println!("  {} {}", group_label.dimmed(), group_id.cyan().bold());
        for ac in acs {
            let status_icon = match ac.status.as_str() {
                "pass" => "✓".green(),
                "fail" => "✗".red(),
                "unknown" => "?".yellow(),
                _ => "·".dimmed(),
            };
            let kernel_marker = if ac.must_have_ac { " 🔒" } else { "" };
            println!("    [{}] {}{}", status_icon, ac.id, kernel_marker);
            println!("         {}", ac.text.dimmed());

            // Suggest next action for non-passing ACs
            if ac.status != "pass" {
                let hint = if ac.status == "unknown" {
                    format!("cargo xtask ac-suggest-scenarios {}", ac.id)
                } else {
                    format!("cargo xtask test-ac {}", ac.id)
                };
                println!("         {} {}", "→".dimmed(), hint.cyan());
            }
        }
        println!();
    }

    // Next steps
    let failing: Vec<_> = filtered.iter().filter(|ac| ac.status == "fail").collect();
    let unknown: Vec<_> = filtered.iter().filter(|ac| ac.status == "unknown").collect();

    if !failing.is_empty() || !unknown.is_empty() {
        println!("{}", "Next Steps:".bold());
        if !failing.is_empty() {
            println!(
                "  1. Fix {} failing AC(s): {}",
                failing.len(),
                format!("cargo xtask test-ac {}", failing[0].id).cyan()
            );
        }
        if !unknown.is_empty() {
            let step = if failing.is_empty() { "1" } else { "2" };
            println!(
                "  {}. Add coverage for {} AC(s): {}",
                step,
                unknown.len(),
                format!("cargo xtask ac-suggest-scenarios {}", unknown[0].id).cyan()
            );
        }
    }

    Ok(())
}

fn describe_filter(args: &AcReportArgs) -> String {
    let mut parts = Vec::new();
    if args.must_have {
        parts.push("must_have_ac=true".to_string());
    }
    if let Some(ref status) = args.status {
        parts.push(format!("status={}", status));
    }
    if parts.is_empty() { "all ACs".to_string() } else { parts.join(", ") }
}

// ===========================================================================
// Markdown output (for PRs)
// ===========================================================================

fn render_markdown(report: &AcReport, args: &AcReportArgs) -> Result<()> {
    let mut out = std::io::stdout();
    render_markdown_to(&mut out, report, args)
}

/// Render markdown to any writer (for testing)
fn render_markdown_to<W: Write>(out: &mut W, report: &AcReport, args: &AcReportArgs) -> Result<()> {
    writeln!(out, "## AC Coverage Report")?;
    writeln!(out)?;
    writeln!(out, "| Category | Total | Pass | Fail | Unknown |")?;
    writeln!(out, "|----------|-------|------|------|---------|")?;
    writeln!(
        out,
        "| Must-have | {} | {} | {} | {} |",
        report.must_have_acs.total,
        report.must_have_acs.passing,
        report.must_have_acs.failing,
        report.must_have_acs.unknown
    )?;
    writeln!(
        out,
        "| Optional | {} | {} | {} | {} |",
        report.optional_acs.total,
        report.optional_acs.passing,
        report.optional_acs.failing,
        report.optional_acs.unknown
    )?;
    writeln!(out)?;
    writeln!(out, "**Coverage:** {:.1}%", report.coverage_percent)?;
    writeln!(out)?;

    // Filter and group
    let filtered = report.filter(args);

    let failing: Vec<_> = filtered.iter().filter(|ac| ac.status == "fail").collect();
    let unknown: Vec<_> = filtered.iter().filter(|ac| ac.status == "unknown").collect();

    if !failing.is_empty() {
        writeln!(out, "### Blockers (Failing ACs)")?;
        writeln!(out)?;
        for ac in &failing {
            writeln!(out, "- **{}** (fail): {}", ac.id, ac.text)?;
            writeln!(out, "  - Requirement: {}", ac.req_id)?;
            writeln!(out, "  - Source: {}", ac.source)?;
        }
        writeln!(out)?;
    }

    if !unknown.is_empty() {
        let title =
            if args.must_have { "### Missing Coverage (Kernel)" } else { "### Missing Coverage" };
        writeln!(out, "{}", title)?;
        writeln!(out)?;
        for ac in &unknown {
            let kernel = if ac.must_have_ac { " 🔒" } else { "" };
            writeln!(out, "- **{}**{}: {}", ac.id, kernel, ac.text)?;
        }
        writeln!(out)?;
    }

    if failing.is_empty() && unknown.is_empty() {
        writeln!(out, "✅ All filtered ACs are passing.")?;
    }

    Ok(())
}

// ===========================================================================
// JSON output (pass-through)
// ===========================================================================

fn render_json(report: &AcReport) -> Result<()> {
    // Re-serialize the report (we could also just re-run ac-status --json,
    // but this proves we parsed it correctly)
    let json = serde_json::to_string_pretty(&report).context("Failed to serialize report")?;
    println!("{}", json);
    Ok(())
}

// Implement Serialize for AcReport to enable JSON output
impl serde::Serialize for AcReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AcReport", 6)?;
        state.serialize_field("schema_version", &self.schema_version)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("must_have_acs", &self.must_have_acs)?;
        state.serialize_field("optional_acs", &self.optional_acs)?;
        state.serialize_field("coverage_percent", &self.coverage_percent)?;
        state.serialize_field("acs", &self.acs)?;
        state.end()
    }
}

impl serde::Serialize for AcCategoryStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AcCategoryStats", 4)?;
        state.serialize_field("total", &self.total)?;
        state.serialize_field("passing", &self.passing)?;
        state.serialize_field("failing", &self.failing)?;
        state.serialize_field("unknown", &self.unknown)?;
        state.end()
    }
}

impl serde::Serialize for AcJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AcJson", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("story_id", &self.story_id)?;
        state.serialize_field("req_id", &self.req_id)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("source", &self.source)?;
        state.serialize_field("must_have_ac", &self.must_have_ac)?;
        state.serialize_field("scenarios", &self.scenarios)?;
        state.serialize_field("tests_total", &self.tests_total)?;
        state.serialize_field("tests_executed", &self.tests_executed)?;
        state.end()
    }
}

// ===========================================================================
// HTML output (for portals)
// ===========================================================================

fn render_html(report: &AcReport, args: &AcReportArgs) -> Result<()> {
    let mut out = std::io::stdout();
    render_html_to(&mut out, report, args)
}

/// Render HTML to any writer (for testing)
fn render_html_to<W: Write>(out: &mut W, report: &AcReport, args: &AcReportArgs) -> Result<()> {
    writeln!(out, "<!DOCTYPE html>")?;
    writeln!(out, "<html><head>")?;
    writeln!(out, "<title>AC Coverage Report</title>")?;
    writeln!(out, "<style>")?;
    writeln!(
        out,
        "body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; }}"
    )?;
    writeln!(out, "table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}")?;
    writeln!(out, "th, td {{ border: 1px solid #ddd; padding: 0.5rem; text-align: left; }}")?;
    writeln!(out, "th {{ background: #f5f5f5; }}")?;
    writeln!(out, ".pass {{ color: #22c55e; }}")?;
    writeln!(out, ".fail {{ color: #ef4444; }}")?;
    writeln!(out, ".unknown {{ color: #eab308; }}")?;
    writeln!(out, ".kernel {{ font-weight: bold; }}")?;
    writeln!(out, "</style>")?;
    writeln!(out, "</head><body>")?;

    writeln!(out, "<h1>AC Coverage Report</h1>")?;
    writeln!(out, "<p>Generated: {}</p>", report.timestamp)?;
    writeln!(out, "<p><strong>Coverage:</strong> {:.1}%</p>", report.coverage_percent)?;

    writeln!(out, "<table>")?;
    writeln!(
        out,
        "<tr><th>Category</th><th>Total</th><th>Pass</th><th>Fail</th><th>Unknown</th></tr>"
    )?;
    writeln!(
        out,
        "<tr><td>Must-have</td><td>{}</td><td class=\"pass\">{}</td><td class=\"fail\">{}</td><td class=\"unknown\">{}</td></tr>",
        report.must_have_acs.total,
        report.must_have_acs.passing,
        report.must_have_acs.failing,
        report.must_have_acs.unknown
    )?;
    writeln!(
        out,
        "<tr><td>Optional</td><td>{}</td><td class=\"pass\">{}</td><td class=\"fail\">{}</td><td class=\"unknown\">{}</td></tr>",
        report.optional_acs.total,
        report.optional_acs.passing,
        report.optional_acs.failing,
        report.optional_acs.unknown
    )?;
    writeln!(out, "</table>")?;

    // Filtered AC list
    let filtered = report.filter(args);
    let groups =
        if args.by_story { report.by_story(&filtered) } else { report.by_requirement(&filtered) };

    writeln!(out, "<h2>ACs by {}</h2>", if args.by_story { "Story" } else { "Requirement" })?;

    for (group_id, acs) in groups {
        writeln!(out, "<h3>{}</h3>", group_id)?;
        writeln!(out, "<ul>")?;
        for ac in acs {
            let status_class = match ac.status.as_str() {
                "pass" => "pass",
                "fail" => "fail",
                "unknown" => "unknown",
                _ => "",
            };
            let kernel_class = if ac.must_have_ac { " kernel" } else { "" };
            writeln!(
                out,
                "<li class=\"{}{}\"><strong>{}</strong>: {} <span class=\"{}\">[{}]</span></li>",
                status_class, kernel_class, ac.id, ac.text, status_class, ac.status
            )?;
        }
        writeln!(out, "</ul>")?;
    }

    writeln!(out, "</body></html>")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
        "schema_version": "2.0",
        "timestamp": "2025-12-05T12:00:00Z",
        "must_have_acs": {"total": 10, "passing": 8, "failing": 1, "unknown": 1},
        "optional_acs": {"total": 5, "passing": 4, "failing": 0, "unknown": 1},
        "coverage_percent": 80.0,
        "acs": [
            {
                "id": "AC-TEST-001",
                "story_id": "US-TEST-001",
                "req_id": "REQ-TEST-001",
                "text": "Test AC 1",
                "status": "pass",
                "source": "coverage",
                "must_have_ac": true,
                "scenarios": ["Test scenario"],
                "tests_total": 1,
                "tests_executed": 1
            },
            {
                "id": "AC-TEST-002",
                "story_id": "US-TEST-001",
                "req_id": "REQ-TEST-001",
                "text": "Test AC 2",
                "status": "unknown",
                "source": "inferred",
                "must_have_ac": true,
                "scenarios": [],
                "tests_total": 1,
                "tests_executed": 0
            },
            {
                "id": "AC-TEST-003",
                "story_id": "US-TEST-002",
                "req_id": "REQ-TEST-002",
                "text": "Test AC 3",
                "status": "fail",
                "source": "coverage",
                "must_have_ac": false,
                "scenarios": ["Failing scenario"],
                "tests_total": 1,
                "tests_executed": 1
            }
        ]
    }"#;

    #[test]
    fn parse_sample_json() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        assert_eq!(report.schema_version, "2.0");
        assert_eq!(report.must_have_acs.total, 10);
        assert_eq!(report.acs.len(), 3);
    }

    #[test]
    fn filter_by_must_have() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs { must_have: true, ..Default::default() };
        let filtered = report.filter(&args);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|ac| ac.must_have_ac));
    }

    #[test]
    fn filter_by_status() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs { status: Some("unknown".to_string()), ..Default::default() };
        let filtered = report.filter(&args);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "AC-TEST-002");
    }

    #[test]
    fn filter_combined() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs {
            must_have: true,
            status: Some("unknown".to_string()),
            ..Default::default()
        };
        let filtered = report.filter(&args);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "AC-TEST-002");
    }

    #[test]
    fn group_by_requirement() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let filtered = report.filter(&AcReportArgs::default());
        let groups = report.by_requirement(&filtered);
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("REQ-TEST-001"));
        assert!(groups.contains_key("REQ-TEST-002"));
    }

    #[test]
    fn group_by_story() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let filtered = report.filter(&AcReportArgs::default());
        let groups = report.by_story(&filtered);
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("US-TEST-001"));
        assert!(groups.contains_key("US-TEST-002"));
    }

    // =========================================================================
    // Golden tests for rendering output
    // =========================================================================

    #[test]
    fn markdown_output_has_expected_structure() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Verify heading
        assert!(output.contains("## AC Coverage Report"), "Missing heading");

        // Verify summary table structure
        assert!(
            output.contains("| Category | Total | Pass | Fail | Unknown |"),
            "Missing table header"
        );
        assert!(output.contains("| Must-have |"), "Missing must-have row");
        assert!(output.contains("| Optional |"), "Missing optional row");

        // Verify coverage percentage
        assert!(output.contains("**Coverage:**"), "Missing coverage line");
        assert!(output.contains("80.0%"), "Coverage value incorrect");
    }

    #[test]
    fn markdown_output_shows_blockers_when_failing() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Should have blockers section for failing AC
        assert!(output.contains("### Blockers"), "Missing blockers section");
        assert!(output.contains("AC-TEST-003"), "Missing failing AC ID");
        assert!(output.contains("(fail)"), "Missing fail status");
    }

    #[test]
    fn markdown_output_shows_missing_coverage() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Should have missing coverage section for unknown AC
        assert!(output.contains("### Missing Coverage"), "Missing coverage section");
        assert!(output.contains("AC-TEST-002"), "Missing unknown AC ID");
    }

    #[test]
    fn markdown_output_shows_all_passing_when_no_issues() {
        // Create a report with all ACs passing
        let json = r#"{
            "schema_version": "2.0",
            "timestamp": "2025-12-05T12:00:00Z",
            "must_have_acs": {"total": 2, "passing": 2, "failing": 0, "unknown": 0},
            "optional_acs": {"total": 0, "passing": 0, "failing": 0, "unknown": 0},
            "coverage_percent": 100.0,
            "acs": [
                {
                    "id": "AC-TEST-001",
                    "story_id": "US-TEST-001",
                    "req_id": "REQ-TEST-001",
                    "text": "Test AC 1",
                    "status": "pass",
                    "source": "coverage",
                    "must_have_ac": true,
                    "scenarios": ["Test scenario"],
                    "tests_total": 1,
                    "tests_executed": 1
                }
            ]
        }"#;

        let report = AcReport::from_json(json).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Should show all passing message
        assert!(output.contains("✅ All filtered ACs are passing"), "Missing all-passing message");
        // Should NOT have blockers section
        assert!(!output.contains("### Blockers"), "Should not have blockers when all passing");
    }

    #[test]
    fn markdown_must_have_filter_shows_kernel_only() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs { must_have: true, ..Default::default() };
        let mut buf = Vec::new();

        render_markdown_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Should include kernel ACs
        assert!(output.contains("AC-TEST-002"), "Missing kernel AC");
        // Should NOT include non-kernel AC (AC-TEST-003 is must_have_ac=false)
        assert!(!output.contains("AC-TEST-003"), "Should not include non-kernel AC");
    }

    #[test]
    fn html_output_has_expected_structure() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_html_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Verify HTML structure
        assert!(output.contains("<!DOCTYPE html>"), "Missing doctype");
        assert!(output.contains("<html>"), "Missing html tag");
        assert!(output.contains("<title>AC Coverage Report</title>"), "Missing title");

        // Verify CSS classes
        assert!(output.contains(".pass {"), "Missing .pass CSS class");
        assert!(output.contains(".fail {"), "Missing .fail CSS class");
        assert!(output.contains(".unknown {"), "Missing .unknown CSS class");
        assert!(output.contains(".kernel {"), "Missing .kernel CSS class");

        // Verify heading
        assert!(output.contains("<h1>AC Coverage Report</h1>"), "Missing h1 heading");
    }

    #[test]
    fn html_output_has_status_classes_on_elements() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_html_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Verify status classes are applied to table cells
        assert!(output.contains("class=\"pass\""), "Missing pass class on element");
        assert!(output.contains("class=\"fail\""), "Missing fail class on element");

        // Verify kernel class is applied
        assert!(output.contains(" kernel\""), "Missing kernel class on element");
    }

    #[test]
    fn html_output_is_self_contained() {
        let report = AcReport::from_json(SAMPLE_JSON).unwrap();
        let args = AcReportArgs::default();
        let mut buf = Vec::new();

        render_html_to(&mut buf, &report, &args).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Should have inline styles, no external dependencies
        assert!(output.contains("<style>"), "Missing inline styles");
        assert!(!output.contains("<script"), "Should not have JavaScript");
        assert!(!output.contains("href=\"http"), "Should not have external links");

        // Should be properly closed
        assert!(output.contains("</body></html>"), "Missing closing tags");
    }
}
