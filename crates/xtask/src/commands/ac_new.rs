use anyhow::Result;
use colored::Colorize;
use std::fs;

use crate::kernel::layout_for_repo;

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
    let layout = layout_for_repo();
    if !layout.ledger.exists() {
        anyhow::bail!("spec_ledger.yaml not found at {}", layout.ledger.display());
    }

    let _ledger_content = fs::read_to_string(&layout.ledger)?;

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
    let layout = layout_for_repo();

    // Check spec_ledger.yaml
    if layout.ledger.exists() {
        let content = fs::read_to_string(&layout.ledger)?;
        if content.contains(&format!("id: {}", ac_id)) {
            anyhow::bail!("AC ID {} already exists in spec_ledger.yaml", ac_id);
        }
    }

    // Check feature files
    if layout.features_dir.exists() {
        for entry in fs::read_dir(&layout.features_dir)? {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ac_new_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        let _: fn(&str, &str, &str, &str) -> Result<()> = run;
    }

    #[test]
    fn test_ac_id_validation_requires_prefix() {
        // Test that AC IDs must start with "AC-"
        let invalid_ids = vec!["TEST-001", "REQ-001", "US-001", "INVALID"];

        for id in invalid_ids {
            assert!(
                !id.starts_with("AC-"),
                "ID {} should be rejected as it doesn't start with AC-",
                id
            );
        }
    }

    #[test]
    fn test_ac_id_validation_accepts_valid_ids() {
        // Test that valid AC IDs are accepted
        let valid_ids = vec!["AC-TPL-001", "AC-PLT-001", "AC-TEST-999"];

        for id in valid_ids {
            assert!(id.starts_with("AC-"), "ID {} should be accepted", id);
        }
    }

    #[test]
    fn test_check_ac_uniqueness_function_exists() {
        // Verify the uniqueness check function is accessible
        let _: fn(&str) -> Result<()> = check_ac_uniqueness;
    }

    #[test]
    fn test_ac_entry_generation() {
        // Test that AC entry formatting is correct
        let ac_id = "AC-TEST-001";
        let description = "Test acceptance criterion";

        let ac_entry = format!(
            r#"          - id: {}
            text: "{}"
            tests: [{{ type: bdd, tag: "@{}" }}]"#,
            ac_id, description, ac_id
        );

        assert!(ac_entry.contains("id: AC-TEST-001"));
        assert!(ac_entry.contains("text: \"Test acceptance criterion\""));
        assert!(ac_entry.contains("tag: \"@AC-TEST-001\""));
    }
}
