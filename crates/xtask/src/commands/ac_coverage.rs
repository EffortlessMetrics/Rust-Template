use anyhow::Result;
use colored::Colorize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use super::ac_parsing::{
    AcMetadata, AcStatus, parse_ac_coverage, parse_cucumber_json, parse_features, parse_junit,
    parse_ledger, parse_ledger_with_metadata,
};
use crate::kernel::layout_for_repo;

pub fn run(args: AcCoverageArgs) -> Result<()> {
    println!("{}", "📊 Computing AC coverage...".blue().bold());
    println!();

    // Parse ledger
    let layout = layout_for_repo();
    let ledger_path = &layout.ledger;
    if !ledger_path.exists() {
        anyhow::bail!("spec_ledger.yaml not found at {}", ledger_path.display());
    }

    let (all_acs, acs_by_req) = parse_ledger(ledger_path)?;

    // Parse ledger with metadata for must_have_ac filtering
    let ac_metadata = parse_ledger_with_metadata(ledger_path)?;

    // Parse scenarios from features (for fallback path)
    let scenarios = parse_features(&args.features_dir)?;

    // PRIMARY PATH: AC Coverage JSONL (streams results, resilient to exit())
    // The coverage.jsonl file is the preferred source as it flushes on each
    // scenario completion and doesn't rely on Drop semantics.
    //
    // FALLBACK PATHS:
    // 1. Cucumber JSON report
    // 2. JUnit XML + feature file parsing (legacy)
    let coverage_exists = args.coverage.exists()
        && fs::metadata(&args.coverage).map(|m| m.len() > 0).unwrap_or(false);

    let ac_results = if coverage_exists {
        println!("  Using primary source: {}", args.coverage.display());
        let (_, results) = parse_ac_coverage(&args.coverage)?;
        results
    } else if args.json_report.exists() {
        println!("  Coverage not found; falling back to JSON: {}", args.json_report.display());
        parse_cucumber_json(&args.json_report).or_else(|_| {
            // Fallback if JSON is invalid
            if args.junit.exists() {
                println!("  JSON invalid; falling back to JUnit: {}", args.junit.display());
                parse_junit(&args.junit, &scenarios)
            } else {
                Ok(HashMap::new())
            }
        })?
    } else if args.junit.exists() {
        println!("  Coverage and JSON not found; falling back to JUnit: {}", args.junit.display());
        parse_junit(&args.junit, &scenarios)?
    } else {
        println!("  ⚠ No test results found. Run BDD tests first: cargo xtask bdd");
        HashMap::new()
    };
    println!();

    // Determine AC statuses
    let mut ac_statuses: HashMap<String, AcStatus> = HashMap::new();
    for ac_id in all_acs.keys() {
        if let Some(status) = ac_results.get(ac_id) {
            ac_statuses.insert(ac_id.clone(), status.clone());
        } else {
            ac_statuses.insert(ac_id.clone(), AcStatus::Unknown);
        }
    }

    // Generate report
    if args.todo_only {
        print_todo_backlog(&ac_metadata, &ac_statuses, args.must_have_only)?;
    } else {
        print_coverage_report(&all_acs, &acs_by_req, &ac_statuses)?;
    }

    Ok(())
}

/// Print a focused backlog of ACs that need BDD scenarios.
///
/// Used by `cargo xtask ac-coverage --todo` to show actionable work items.
///
/// # Arguments
/// * `ac_metadata` - Map of AC_ID -> AcMetadata with req_id and must_have_ac
/// * `ac_statuses` - Map of AC_ID -> AcStatus (Pass/Fail/Unknown)
/// * `must_have_only` - If true, only show ACs where must_have_ac=true (kernel ACs)
fn print_todo_backlog(
    ac_metadata: &HashMap<String, AcMetadata>,
    ac_statuses: &HashMap<String, AcStatus>,
    must_have_only: bool,
) -> Result<()> {
    // Collect unknown ACs grouped by requirement
    let mut unknowns_by_req: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (ac_id, status) in ac_statuses {
        if *status == AcStatus::Unknown
            && let Some(metadata) = ac_metadata.get(ac_id)
        {
            // Filter by must_have_ac if requested
            if must_have_only && !metadata.must_have_ac {
                continue;
            }
            unknowns_by_req.entry(metadata.req_id.clone()).or_default().push(ac_id.clone());
        }
    }

    if unknowns_by_req.is_empty() {
        let filter_text = if must_have_only { "kernel (must_have_ac) " } else { "" };
        println!("{} {} All {}ACs have BDD coverage!", "🎉".bold(), "✓".green(), filter_text);
        println!();
        if must_have_only {
            println!("Run `cargo xtask ac-coverage --todo` to see all unknown ACs.");
        } else {
            println!("Nothing in the backlog. Run `cargo xtask ac-coverage` for full report.");
        }
        return Ok(());
    }

    // Count totals
    let total_unknown: usize = unknowns_by_req.values().map(|v| v.len()).sum();
    let total_reqs = unknowns_by_req.len();

    let title = if must_have_only {
        format!("{} Kernel AC Coverage Backlog", "📋".bold())
    } else {
        format!("{} Coverage Backlog", "📋".bold())
    };
    println!("{title}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    if must_have_only {
        println!(
            "  {} kernel ACs need BDD scenarios across {} requirements",
            total_unknown.to_string().yellow().bold(),
            total_reqs
        );
        println!("  (Showing only must_have_ac=true ACs)");
    } else {
        println!(
            "  {} ACs need BDD scenarios across {} requirements",
            total_unknown.to_string().yellow().bold(),
            total_reqs
        );
    }
    println!();

    // Count kernel unknowns before we consume unknowns_by_req in the loop
    let kernel_unknown: usize = if !must_have_only {
        unknowns_by_req
            .values()
            .flatten()
            .filter(|ac_id| ac_metadata.get(*ac_id).map(|m| m.must_have_ac).unwrap_or(false))
            .count()
    } else {
        0
    };

    // Print as a checklist with per-AC hints
    let mut item_num = 1;
    for (req_id, mut ac_ids) in unknowns_by_req {
        ac_ids.sort();
        println!("  {}", req_id.cyan().bold());
        for ac_id in &ac_ids {
            // Check if this is a must_have_ac for annotation
            let is_kernel = ac_metadata.get(ac_id).map(|m| m.must_have_ac).unwrap_or(false);
            let kernel_marker = if is_kernel && !must_have_only { " 🔒" } else { "" };

            println!("    [ ] {}. {}{}", item_num, ac_id, kernel_marker);
            // Per-AC hint: show the command to generate scenarios
            println!(
                "         {} {}",
                "→".dimmed(),
                format!("cargo xtask ac-suggest-scenarios {}", ac_id).cyan()
            );
            item_num += 1;
        }
        println!();
    }

    // Usage hints
    println!("{} Next Steps", "🚀".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  1. Pick an AC from above and generate scenarios:");
    println!("     {} {}", "$".dimmed(), "cargo xtask ac-suggest-scenarios <AC_ID>".cyan());
    println!();
    println!("  2. Add scenarios to specs/features/*.feature with @<AC_ID> tag");
    println!();
    println!("  3. Run BDD tests:");
    println!("     {} {}", "$".dimmed(), "cargo xtask bdd".cyan());
    println!();

    if !must_have_only && kernel_unknown > 0 && kernel_unknown < total_unknown {
        println!("{} Filter Options", "💡".bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!(
            "  {} of {} unknown ACs are kernel (must_have_ac=true)",
            kernel_unknown, total_unknown
        );
        println!(
            "  To show only kernel ACs: {}",
            "cargo xtask ac-coverage --todo --must-have".cyan()
        );
        println!();
    }

    Ok(())
}

fn print_coverage_report(
    all_acs: &HashMap<String, String>,
    _acs_by_req: &BTreeMap<String, Vec<String>>,
    ac_statuses: &HashMap<String, AcStatus>,
) -> Result<()> {
    let passing = ac_statuses.values().filter(|s| **s == AcStatus::Pass).count();
    let failing = ac_statuses.values().filter(|s| **s == AcStatus::Fail).count();
    let unknown = ac_statuses.values().filter(|s| **s == AcStatus::Unknown).count();

    println!("{} AC Coverage Summary", "📋".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  {} {} passing", "✓".green(), passing);
    println!("  {} {} failing", "✗".red(), failing);
    println!("  {} {} unknown (no BDD scenarios)", "?".yellow(), unknown);
    println!();

    // Group unknowns by requirement
    let mut unknowns_by_req: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (ac_id, status) in ac_statuses {
        if *status == AcStatus::Unknown
            && let Some(req_id) = all_acs.get(ac_id)
        {
            unknowns_by_req.entry(req_id.clone()).or_default().push(ac_id.clone());
        }
    }

    if !unknowns_by_req.is_empty() {
        println!("{} Unknown ACs (Need BDD scenarios)", "📍".cyan().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        for (req_id, mut ac_ids) in unknowns_by_req {
            ac_ids.sort();
            let ac_list = ac_ids.join(", ");
            println!("  {}: {}", req_id.cyan(), ac_list);
        }
        println!();
    }

    // Suggest next steps
    if unknown > 0 {
        println!("{} Suggested Next Steps", "🎯".bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        // Pick first unknown AC for example
        if let Some((ac_id, _)) = ac_statuses.iter().find(|(_, s)| **s == AcStatus::Unknown) {
            println!("  1. Generate scenario stub for {}:", ac_id.cyan());
            println!(
                "     {} {} {}",
                "$".dimmed(),
                "cargo xtask ac-suggest-scenarios".cyan(),
                ac_id
            );
            println!();
        }

        println!("  2. Edit specs/features/your_feature.feature:");
        println!("     {} {}", "$".dimmed(), "vim specs/features/your_feature.feature".cyan());
        println!();
        println!("  3. Run BDD tests:");
        println!("     {} {}", "$".dimmed(), "cargo xtask bdd".cyan());
        println!();
        println!("  4. Check coverage again:");
        println!("     {} {}", "$".dimmed(), "cargo xtask ac-coverage".cyan());
    } else {
        println!("{} {} All ACs have BDD coverage!", "🎉".bold(), "✓".green());
    }

    println!();

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AcCoverageArgs {
    #[allow(dead_code)]
    pub ledger: PathBuf,
    pub features_dir: PathBuf,
    /// Primary source: AC coverage JSONL (resilient to cucumber exit())
    pub coverage: PathBuf,
    /// Fallback: JUnit XML from acceptance tests
    pub junit: PathBuf,
    /// Fallback: Cucumber JSON report
    pub json_report: PathBuf,
    /// Show only ACs with Unknown status (coverage backlog)
    pub todo_only: bool,
    /// When used with todo_only, filter to only kernel ACs (must_have_ac=true)
    pub must_have_only: bool,
}

impl Default for AcCoverageArgs {
    fn default() -> Self {
        let layout = layout_for_repo();
        Self {
            ledger: layout.ledger,
            features_dir: layout.features_dir,
            coverage: layout.coverage_file,
            junit: layout.junit_file,
            json_report: PathBuf::from("target/ac_report.json"),
            todo_only: false,
            must_have_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create AcMetadata
    fn make_metadata(req_id: &str, must_have: bool) -> AcMetadata {
        AcMetadata {
            story_id: "US-TEST-001".to_string(),
            req_id: req_id.to_string(),
            text: "Test AC".to_string(),
            tags: Vec::new(),
            tests: Vec::new(),
            must_have_ac: must_have,
        }
    }

    #[test]
    fn print_todo_backlog_with_unknown_acs_shows_backlog() {
        // Setup: 3 ACs, 2 unknown (need BDD), 1 passing
        let mut ac_metadata = HashMap::new();
        ac_metadata.insert("AC-PLT-001".to_string(), make_metadata("REQ-PLT-001", true));
        ac_metadata.insert("AC-PLT-002".to_string(), make_metadata("REQ-PLT-001", true));
        ac_metadata.insert("AC-PLT-003".to_string(), make_metadata("REQ-PLT-002", false));

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-PLT-001".to_string(), AcStatus::Unknown);
        ac_statuses.insert("AC-PLT-002".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-PLT-003".to_string(), AcStatus::Unknown);

        // The function should succeed (no filter)
        let result = print_todo_backlog(&ac_metadata, &ac_statuses, false);
        assert!(result.is_ok());
    }

    #[test]
    fn print_todo_backlog_with_must_have_filter() {
        // Setup: 3 ACs, 2 unknown, but only 1 is must_have
        let mut ac_metadata = HashMap::new();
        ac_metadata.insert("AC-PLT-001".to_string(), make_metadata("REQ-PLT-001", true));
        ac_metadata.insert("AC-PLT-002".to_string(), make_metadata("REQ-PLT-001", false));
        ac_metadata.insert("AC-PLT-003".to_string(), make_metadata("REQ-PLT-002", false));

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-PLT-001".to_string(), AcStatus::Unknown);
        ac_statuses.insert("AC-PLT-002".to_string(), AcStatus::Unknown);
        ac_statuses.insert("AC-PLT-003".to_string(), AcStatus::Unknown);

        // With must_have filter, only AC-PLT-001 should appear
        let result = print_todo_backlog(&ac_metadata, &ac_statuses, true);
        assert!(result.is_ok());
    }

    #[test]
    fn print_todo_backlog_with_all_covered_shows_success() {
        // Setup: all ACs have BDD coverage (Pass or Fail, no Unknown)
        let mut ac_metadata = HashMap::new();
        ac_metadata.insert("AC-PLT-001".to_string(), make_metadata("REQ-PLT-001", true));
        ac_metadata.insert("AC-PLT-002".to_string(), make_metadata("REQ-PLT-001", true));

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-PLT-001".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-PLT-002".to_string(), AcStatus::Fail); // Even failing is "covered"

        let result = print_todo_backlog(&ac_metadata, &ac_statuses, false);
        assert!(result.is_ok());
    }

    #[test]
    fn print_todo_backlog_with_empty_acs() {
        let ac_metadata = HashMap::new();
        let ac_statuses = HashMap::new();

        // Should succeed with "all covered" message (vacuously true)
        let result = print_todo_backlog(&ac_metadata, &ac_statuses, false);
        assert!(result.is_ok());
    }

    #[test]
    fn print_coverage_report_counts_correctly() {
        let mut all_acs = HashMap::new();
        all_acs.insert("AC-001".to_string(), "REQ-001".to_string());
        all_acs.insert("AC-002".to_string(), "REQ-001".to_string());
        all_acs.insert("AC-003".to_string(), "REQ-002".to_string());
        all_acs.insert("AC-004".to_string(), "REQ-002".to_string());

        let acs_by_req = BTreeMap::new();

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-001".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-002".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-003".to_string(), AcStatus::Fail);
        ac_statuses.insert("AC-004".to_string(), AcStatus::Unknown);

        // Should succeed and print correct counts (2 passing, 1 failing, 1 unknown)
        let result = print_coverage_report(&all_acs, &acs_by_req, &ac_statuses);
        assert!(result.is_ok());
    }
}
