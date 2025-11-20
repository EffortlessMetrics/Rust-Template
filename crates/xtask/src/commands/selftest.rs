use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Run full template self-test suite
#[allow(dead_code)]
pub fn run() -> Result<()> {
    run_with_verbosity(crate::Verbosity::Normal)
}

/// Run full template self-test suite with specified verbosity
pub fn run_with_verbosity(verbosity: crate::Verbosity) -> Result<()> {
    let start_time = Instant::now();
    println!("{}", "======================================".blue());
    println!("{}", "  Template Self-Test Suite".blue());
    println!("{}", "======================================".blue());
    println!();

    let mut failed = 0;

    // Step 1: Core checks
    println!("{}", "[1/5] Running core checks (fmt, clippy, tests)...".blue());
    let step_start = Instant::now();
    match crate::commands::check::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Core checks passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Core checks passed", "✓".green());
            }
        }
        Err(e) => {
            eprintln!("  {} Core checks failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 2: BDD acceptance tests
    println!("{}", "[2/5] Running BDD acceptance tests...".blue());
    let step_start = Instant::now();
    match crate::commands::bdd::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} BDD scenarios passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} BDD scenarios passed", "✓".green());
            }
            if Path::new("target/junit/acceptance.xml").exists() {
                println!("  {} JUnit XML generated", "✓".green());
            } else {
                println!("  {} JUnit XML not found", "⚠".yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} BDD tests failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 3: AC status mapping & ADR references
    println!("{}", "[3/5] Running AC status mapping & ADR references...".blue());
    let step_start = Instant::now();

    // 3a: AC status
    match run_ac_status(verbosity) {
        Ok(_) => {
            if verbosity.is_verbose() {
                println!("  {} AC status script executed", "✓".green());
            }
            if Path::new("docs/feature_status.md").exists() {
                println!("  {} Feature status generated", "✓".green());
            } else {
                println!("  {} Feature status not found", "⚠".yellow());
            }
        }
        Err(e) => {
            eprintln!("  {} AC status failed: {}", "✗".red(), e);
            // Don't fail the suite if AC status has issues - it's informational
            println!("  {} Continuing (AC status is informational)", "⚠".yellow());
        }
    }

    // 3b: ADR references
    match run_adr_check(verbosity) {
        Ok(_) => {
            println!("  {} ADR references validated", "✓".green());
        }
        Err(e) => {
            eprintln!("  {} ADR check failed: {}", "✗".red(), e);
            failed += 1;
        }
    }

    let elapsed = step_start.elapsed();
    if verbosity.is_verbose() {
        println!("  {} Step 3 completed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
    }
    println!();

    // Step 4: LLM context bundler
    println!("{}", "[4/6] Testing LLM context bundler...".blue());
    let step_start = Instant::now();
    match crate::commands::bundle::run("implement_ac") {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Bundle generated ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Bundle generated", "✓".green());
            }
            if let Ok(metadata) = std::fs::metadata(".llm/bundle/implement_ac.md") {
                println!("  {} Bundle size: {} bytes", "✓".green(), metadata.len());
            }
        }
        Err(e) => {
            eprintln!("  {} Bundler failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 5: Policy tests (if conftest available)
    println!("{}", "[5/6] Running policy tests...".blue());
    let step_start = Instant::now();
    match crate::commands::policy_test::run() {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Policy tests passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Policy tests passed", "✓".green());
            }
        }
        Err(e) => {
            // Check if this is a "conftest not found" error
            let is_conftest_not_found =
                matches!(e, crate::commands::policy_test::PolicyTestError::ConftestNotFound(_));

            if is_conftest_not_found {
                // In CI, treat this as a failure; locally, just warn
                let is_ci = std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();

                if is_ci {
                    eprintln!(
                        "  {} Policy tests: conftest not found (CI requires conftest)",
                        "✗".red()
                    );
                    if verbosity.is_verbose() {
                        eprintln!("\n{}", e);
                    }
                    failed += 1;
                } else {
                    println!("  {} Policy tests skipped: conftest not found", "⚠".yellow());

                    // Check if nix is available and provide helpful hint
                    if which::which("nix").is_ok() {
                        println!(
                            "  💡 Hint: Run {} for full validation",
                            "nix develop -c cargo run -p xtask -- selftest".cyan()
                        );
                    } else {
                        println!(
                            "  💡 For full policy testing, see: {}",
                            "docs/dev-environment.md".cyan()
                        );
                    }

                    if verbosity.is_verbose() {
                        println!("\n{}", e);
                    }
                    // Don't increment failed counter for local development
                }
            } else {
                eprintln!("  {} Policy tests: {}", "✗".red(), e);
                failed += 1;
            }
        }
    }
    // Step 6: DevEx contract
    println!("{}", "[6/6] Checking DevEx contract...".blue());
    let step_start = Instant::now();
    match run_devex_contract(verbosity) {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!(
                    "  {} DevEx contract satisfied ({:.2}s)",
                    "✓".green(),
                    elapsed.as_secs_f64()
                );
            } else {
                println!("  {} DevEx contract satisfied", "✓".green());
            }
        }
        Err(e) => {
            eprintln!("  {} DevEx contract failed: {}", "✗".red(), e);
            failed += 1;
        }
    }
    println!();

    // Step 7: Graph invariants
    println!("{}", "[7/7] Checking governance graph invariants...".blue());
    let step_start = Instant::now();
    match crate::commands::graph_export::run_graph_invariants(verbosity.as_u8()) {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!(
                    "  {} Graph invariants satisfied ({:.2}s)",
                    "✓".green(),
                    elapsed.as_secs_f64()
                );
            } else {
                println!("  {} Graph invariants satisfied", "✓".green());
            }
        }
        Err(e) => {
            // The error message from run_graph_invariants already includes the violations list
            // but we want to format the header nicely
            eprintln!("  {} Graph invariants failed:", "✗".red());
            eprintln!("{}", e);
            failed += 1;
        }
    }
    println!();

    // Summary
    let total_elapsed = start_time.elapsed();
    println!("{}", "======================================".blue());
    if failed == 0 {
        println!("{}", "✓ All self-tests passed!".green());
        if verbosity.is_verbose() {
            println!("\n{} {:.2}s", "Total elapsed time:".bold(), total_elapsed.as_secs_f64());
        }
        println!();
        println!("The template is working correctly:");
        println!("  • xtask commands functional");
        println!("  • BDD scenarios passing");
        println!("  • AC mapping operational");
        println!("  • LLM bundler working");
        println!();
        println!("Ready for:");
        println!("  • Service development: {}", "docs/how-to/new-service-from-template.md".blue());
        println!("  • AC-first workflow: {}", "docs/tutorials/first-ac-change.md".blue());
        println!("{}", "======================================".blue());
        Ok(())
    } else {
        eprintln!("{}", format!("✗ {} test suite(s) failed", failed).red());
        if verbosity.is_verbose() {
            eprintln!("\n{} {:.2}s", "Total elapsed time:".bold(), total_elapsed.as_secs_f64());
        }
        println!("{}", "======================================".blue());
        anyhow::bail!("{} test suites failed", failed)
    }
}

fn run_ac_status(verbosity: crate::Verbosity) -> Result<()> {
    // Use Rust-native AC status implementation
    crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity,
        ..Default::default()
    })
}

fn run_adr_check(verbosity: crate::Verbosity) -> Result<()> {
    crate::commands::adr_check::run(crate::commands::adr_check::AdrCheckArgs {
        verbosity,
        ..Default::default()
    })
}

fn run_devex_contract(_verbosity: crate::Verbosity) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let spec_path = root.join("specs/devex_flows.yaml");
    let spec = crate::devex::load_spec(&spec_path)?;

    // Get list of available commands
    let available_commands = crate::all_command_names();

    let mut missing = Vec::new();
    for (cmd_name, cmd_spec) in &spec.commands {
        if cmd_spec.required && !available_commands.contains(&cmd_name.as_str()) {
            missing.push(cmd_name.clone());
        }
    }

    if !missing.is_empty() {
        eprintln!();
        eprintln!("✗ Required commands missing from xtask:");
        for name in &missing {
            eprintln!("  • {}", name);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Implement missing command(s) in crates/xtask");
        eprintln!("  • Or update specs/devex_flows.yaml if spec is outdated");
        anyhow::bail!("DevEx contract: {} required command(s) missing", missing.len());
    }

    Ok(())
}
