use anyhow::{Context, Result};
use std::path::Path;

/// Run tests for a specific acceptance criterion
pub fn run(ac_id: &str) -> Result<()> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .context("Failed to determine workspace root")?;

    let ledger_path = workspace_root.join("specs/spec_ledger.yaml");
    let features_dir = workspace_root.join("specs/features");

    // Validate AC exists in ledger
    println!("[INFO] Looking up AC: {}", ac_id);
    let (all_acs, _) = super::ac_parsing::parse_ledger(&ledger_path)?;

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
    let scenarios = super::ac_parsing::parse_features_with_metadata(&features_dir)?;

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
    let test_result = crate::run_cmd(&mut crate::cargo_cmd(
        "test",
        &["-p", "acceptance", "--test", "acceptance", "--", "--tags", &tag_filter],
    ));

    match test_result {
        Ok(_) => {
            println!("\n[PASS] All tests passed for {}", ac_id);
            println!("       Scenarios: {}", ac_scenarios.len());
            Ok(())
        }
        Err(e) => {
            println!("\n[FAIL] Tests failed for {}", ac_id);
            println!("       Scenarios: {}", ac_scenarios.len());
            Err(e).context(format!("Tests failed for {}", ac_id))
        }
    }
}
