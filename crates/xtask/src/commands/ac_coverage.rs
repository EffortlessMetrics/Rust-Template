use anyhow::Result;
use colored::Colorize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::path::PathBuf;

use super::ac_parsing::{AcStatus, parse_cucumber_json, parse_features, parse_junit, parse_ledger};

pub fn run(args: AcCoverageArgs) -> Result<()> {
    println!("{}", "📊 Computing AC coverage...".blue().bold());
    println!();

    // Parse ledger
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        anyhow::bail!("spec_ledger.yaml not found at {}", ledger_path.display());
    }

    let (all_acs, acs_by_req) = parse_ledger(ledger_path)?;

    // Parse scenarios from features
    let scenarios = parse_features(&args.features_dir)?;

    // Parse test results
    let ac_results = if args.json_report.exists() {
        parse_cucumber_json(&args.json_report).or_else(|_| {
            // Fallback if JSON doesn't exist or is invalid
            if args.junit.exists() {
                parse_junit(&args.junit, &scenarios)
            } else {
                Ok(HashMap::new())
            }
        })?
    } else if args.junit.exists() {
        parse_junit(&args.junit, &scenarios)?
    } else {
        HashMap::new()
    };

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
    print_coverage_report(&all_acs, &acs_by_req, &ac_statuses)?;

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
    pub junit: PathBuf,
    pub json_report: PathBuf,
}

impl Default for AcCoverageArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            features_dir: PathBuf::from("specs/features"),
            junit: PathBuf::from("target/junit/acceptance.xml"),
            json_report: PathBuf::from("target/ac_report.json"),
        }
    }
}
