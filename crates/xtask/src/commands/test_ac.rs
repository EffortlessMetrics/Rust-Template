use anyhow::{Context, Result};

use crate::kernel::layout_for_repo;

/// Run tests for a specific acceptance criterion
pub fn run(ac_id: &str) -> Result<()> {
    let layout = layout_for_repo();

    // Validate AC exists in ledger
    println!("[INFO] Looking up AC: {}", ac_id);
    let (all_acs, _) = super::ac_parsing::parse_ledger(&layout.ledger)?;

    if !all_acs.contains_key(ac_id) {
        let preview =
            all_acs.keys().take(10).map(|k| format!("  - {}", k)).collect::<Vec<_>>().join("\n");

        anyhow::bail!(
            "AC '{}' not found in spec_ledger.yaml. Available ACs (first 10):\n{}",
            ac_id,
            preview
        );
    }

    let req_id = all_acs.get(ac_id).unwrap();
    println!("[INFO] Found AC: {} (requirement: {})", ac_id, req_id);

    // Find scenarios tagged with this AC
    println!("\n[INFO] Searching for BDD scenarios tagged with @{}...", ac_id);
    let scenarios = super::ac_parsing::parse_features_with_metadata(&layout.features_dir)?;

    let ac_scenarios: Vec<_> = scenarios.values().filter(|s| s.ac_id == ac_id).collect();

    if ac_scenarios.is_empty() {
        println!("[WARN] No BDD scenarios found for {}", ac_id);
        println!("       Consider running: cargo xtask ac-suggest-scenarios {}", ac_id);
        return Ok(());
    }

    println!("[INFO] Found {} scenario(s):", ac_scenarios.len());
    for scenario in &ac_scenarios {
        println!("  - {} ({})", scenario.name, scenario.file);
    }

    // Run BDD tests filtered by AC tag
    println!("\n[INFO] Running acceptance tests for @{}...", ac_id);

    let tag_filter = format!("@{}", ac_id);
    let mut cmd = crate::cargo_cmd("test", &["-p", "acceptance", "--test", "acceptance"]);
    cmd.env("CUCUMBER_TAG_EXPRESSION", &tag_filter);
    let output = cmd.output().context("Failed to run acceptance tests")?;

    // Use semantic BDD success detection (not just exit code)
    if super::bdd::is_bdd_success(&output) {
        println!("\n[PASS] All tests passed for {}", ac_id);
        println!("       Scenarios: {}", ac_scenarios.len());
        Ok(())
    } else {
        // Print output for debugging
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stdout.is_empty() {
            eprintln!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        println!("\n[FAIL] Tests failed for {}", ac_id);
        println!("       Scenarios: {}", ac_scenarios.len());
        anyhow::bail!("Tests failed for {}", ac_id)
    }
}
