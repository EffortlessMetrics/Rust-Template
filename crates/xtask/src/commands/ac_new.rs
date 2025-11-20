use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run(ac_id: &str, description: &str, _story: &str, requirement: &str) -> Result<()> {
    println!("{}", "✨ Creating new acceptance criterion...".blue().bold());
    println!();

    // Validate AC ID format
    if !ac_id.starts_with("AC-") {
        anyhow::bail!("AC ID must start with 'AC-' (e.g., AC-TPL-001)");
    }

    // Check AC ID uniqueness
    check_ac_uniqueness(ac_id)?;

    // Find the requirement in spec_ledger
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        anyhow::bail!("spec_ledger.yaml not found at {}", ledger_path.display());
    }

    let _ledger_content = fs::read_to_string(ledger_path)?;

    // Simple insertion: find the requirement and add AC
    // This is a basic implementation - more sophisticated YAML parsing could be added
    let ac_entry = format!(
        r#"          - id: {}
            text: "{}"
            tests: [{{ type: bdd, tag: "@{}" }}]"#,
        ac_id, description, ac_id
    );

    println!("{} Prepared AC entry for {} under {}", "✓".green(), ac_id, requirement);
    println!();
    println!("{}", "AC Entry (add to spec_ledger.yaml):".bold());
    println!("{}", ac_entry.dimmed());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Find requirement {} in specs/spec_ledger.yaml", requirement.cyan());
    println!("  2. Add the AC under acceptance_criteria:");
    println!("{}", ac_entry.dimmed());
    println!("  3. Add scenario to specs/features/*.feature:");
    println!();
    println!("     {}", format!("@{}", ac_id).cyan());
    println!("     Scenario: {}", description);
    println!("       Given ...");
    println!("       When ...");
    println!("       Then ...");
    println!();
    println!("  4. Run: {}", "cargo xtask ac-status".cyan());
    println!("  5. Run: {}", "cargo xtask selftest".cyan());

    Ok(())
}

fn check_ac_uniqueness(ac_id: &str) -> Result<()> {
    // Check spec_ledger.yaml
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    if ledger_path.exists() {
        let content = fs::read_to_string(ledger_path)?;
        if content.contains(&format!("id: {}", ac_id)) {
            anyhow::bail!("AC ID {} already exists in spec_ledger.yaml", ac_id);
        }
    }

    // Check feature files
    let features_dir = Path::new("specs/features");
    if features_dir.exists() {
        for entry in fs::read_dir(features_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("feature") {
                let content = fs::read_to_string(&path)?;
                if content.contains(&format!("@{}", ac_id)) {
                    anyhow::bail!("AC tag @{} already exists in {}", ac_id, path.display());
                }
            }
        }
    }

    Ok(())
}
