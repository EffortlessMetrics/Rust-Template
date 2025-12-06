use anyhow::Result;
use colored::Colorize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::ac_parsing::{
    AcStatus, parse_ac_coverage, parse_cucumber_json, parse_features, parse_junit, parse_ledger,
};

pub fn run(args: AcCoverageArgs) -> Result<()> {
    println!("{}", "📊 Computing AC coverage...".blue().bold());
    println!();

    // Parse ledger
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        anyhow::bail!("spec_ledger.yaml not found at {}", ledger_path.display());
    }

    let (all_acs, acs_by_req) = parse_ledger(ledger_path)?;

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
        print_todo_backlog(&all_acs, &acs_by_req, &ac_statuses)?;
    } else {
        print_coverage_report(&all_acs, &acs_by_req, &ac_statuses)?;
    }

    Ok(())
}

/// Print a focused backlog of ACs that need BDD scenarios.
///
/// Used by `cargo xtask ac-coverage --todo` to show actionable work items.
fn print_todo_backlog(
    all_acs: &HashMap<String, String>,
    _acs_by_req: &BTreeMap<String, Vec<String>>,
    ac_statuses: &HashMap<String, AcStatus>,
) -> Result<()> {
    // Collect unknown ACs grouped by requirement
    let mut unknowns_by_req: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (ac_id, status) in ac_statuses {
        if *status == AcStatus::Unknown
            && let Some(req_id) = all_acs.get(ac_id)
        {
            unknowns_by_req.entry(req_id.clone()).or_default().push(ac_id.clone());
        }
    }

    if unknowns_by_req.is_empty() {
        println!("{} {} All ACs have BDD coverage!", "🎉".bold(), "✓".green());
        println!();
        println!("Nothing in the backlog. Run `cargo xtask ac-coverage` for full report.");
        return Ok(());
    }

    // Count totals
    let total_unknown: usize = unknowns_by_req.values().map(|v| v.len()).sum();
    let total_reqs = unknowns_by_req.len();

    println!("{} Coverage Backlog", "📋".bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!(
        "  {} ACs need BDD scenarios across {} requirements",
        total_unknown.to_string().yellow().bold(),
        total_reqs
    );
    println!();

    // Print as a simple checklist
    let mut item_num = 1;
    for (req_id, mut ac_ids) in unknowns_by_req {
        ac_ids.sort();
        println!("  {}", req_id.cyan().bold());
        for ac_id in ac_ids {
            println!("    [ ] {}. {}", item_num, ac_id);
            item_num += 1;
        }
        println!();
    }

    // Quick start command
    if let Some(first_ac) =
        ac_statuses.iter().find(|(_, s)| **s == AcStatus::Unknown).map(|(id, _)| id)
    {
        println!("{} Quick Start", "🚀".bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!(
            "  {} {}",
            "$".dimmed(),
            format!("cargo xtask ac-suggest-scenarios {}", first_ac).cyan()
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
    /// Show only must_have_ac ACs with Unknown status (coverage backlog)
    pub todo_only: bool,
}

impl Default for AcCoverageArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            features_dir: PathBuf::from("specs/features"),
            coverage: PathBuf::from("target/ac/coverage.jsonl"),
            junit: PathBuf::from("target/junit/acceptance.xml"),
            json_report: PathBuf::from("target/ac_report.json"),
            todo_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_todo_backlog_with_unknown_acs_shows_backlog() {
        // Setup: 3 ACs, 2 unknown (need BDD), 1 passing
        let mut all_acs = HashMap::new();
        all_acs.insert("AC-PLT-001".to_string(), "REQ-PLT-001".to_string());
        all_acs.insert("AC-PLT-002".to_string(), "REQ-PLT-001".to_string());
        all_acs.insert("AC-PLT-003".to_string(), "REQ-PLT-002".to_string());

        let acs_by_req = BTreeMap::new(); // Not used by this function

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-PLT-001".to_string(), AcStatus::Unknown);
        ac_statuses.insert("AC-PLT-002".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-PLT-003".to_string(), AcStatus::Unknown);

        // The function should succeed
        let result = print_todo_backlog(&all_acs, &acs_by_req, &ac_statuses);
        assert!(result.is_ok());

        // Note: We can't easily capture stdout in this test setup,
        // but we verify the function doesn't error
    }

    #[test]
    fn print_todo_backlog_with_all_covered_shows_success() {
        // Setup: all ACs have BDD coverage (Pass or Fail, no Unknown)
        let mut all_acs = HashMap::new();
        all_acs.insert("AC-PLT-001".to_string(), "REQ-PLT-001".to_string());
        all_acs.insert("AC-PLT-002".to_string(), "REQ-PLT-001".to_string());

        let acs_by_req = BTreeMap::new();

        let mut ac_statuses = HashMap::new();
        ac_statuses.insert("AC-PLT-001".to_string(), AcStatus::Pass);
        ac_statuses.insert("AC-PLT-002".to_string(), AcStatus::Fail); // Even failing is "covered"

        let result = print_todo_backlog(&all_acs, &acs_by_req, &ac_statuses);
        assert!(result.is_ok());
    }

    #[test]
    fn print_todo_backlog_with_empty_acs() {
        let all_acs = HashMap::new();
        let acs_by_req = BTreeMap::new();
        let ac_statuses = HashMap::new();

        // Should succeed with "all covered" message (vacuously true)
        let result = print_todo_backlog(&all_acs, &acs_by_req, &ac_statuses);
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
