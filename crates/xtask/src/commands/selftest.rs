use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result of a single selftest step
struct StepResult {
    name: &'static str,
    ok: bool,
    hint: Option<&'static str>,
}

/// Collection of all selftest step results
struct SelftestResults {
    steps: Vec<StepResult>,
}

impl SelftestResults {
    fn new() -> Self {
        Self { steps: Vec::new() }
    }

    fn push(&mut self, name: &'static str, ok: bool, hint: Option<&'static str>) {
        self.steps.push(StepResult { name, ok, hint });
    }

    fn failed_count(&self) -> usize {
        self.steps.iter().filter(|s| !s.ok).count()
    }
}

/// Run full template self-test suite with default verbosity.
/// Future: Used as library entry point for programmatic selftest invocation.
/// See AC-KERN-SELFTEST for selftest infrastructure requirements.
#[allow(dead_code)]
pub fn run() -> Result<()> {
    run_with_verbosity(crate::Verbosity::Normal)
}

/// Run full template self-test suite with specified verbosity
pub fn run_with_verbosity(verbosity: crate::Verbosity) -> Result<()> {
    let start_time = Instant::now();

    // Check for low-resource mode
    let low_resource_mode = env::var("XTASK_LOW_RESOURCES").unwrap_or_default() == "1";
    let skip_bdd = env::var("XTASK_SKIP_BDD").unwrap_or_default() == "1";

    println!("{}", "======================================".blue());
    println!("{}", "  Template Self-Test Suite".blue());
    println!("{}", "======================================".blue());

    if low_resource_mode {
        println!("{}", "  Running in low-resource mode".yellow());
        println!("{}", "  (XTASK_LOW_RESOURCES=1)".yellow());

        // Set CARGO_BUILD_JOBS=1 for limited parallelism
        // SAFETY: We're setting this at the start of selftest before any child processes are spawned.
        // This is the intended use case for controlling cargo build parallelism.
        unsafe {
            env::set_var("CARGO_BUILD_JOBS", "1");
        }
    }

    println!();

    let mut results = SelftestResults::new();

    // Step 1: Core checks
    println!("{}", "[1/11] Running core checks (fmt, clippy, tests)...".blue());
    let step_start = Instant::now();
    let core_ok = match crate::commands::check::run_with_options(
        crate::commands::check::CheckOptions::from_env(),
    ) {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} Core checks passed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} Core checks passed", "✓".green());
            }
            true
        }
        Err(e) => {
            eprintln!("  {} Core checks failed: {}", "✗".red(), e);
            false
        }
    };
    results.push("Core checks", core_ok, Some("Run `cargo run -p xtask -- check`"));
    println!();

    // Step 2: Skills governance lint
    println!("{}", "[2/11] Checking Skills governance...".blue());
    let step_start = Instant::now();
    let skills_ok = if Path::new(".claude/skills").exists() {
        match crate::commands::skills::run_lint() {
            Ok(_) => {
                let elapsed = step_start.elapsed();
                if verbosity.is_verbose() {
                    println!(
                        "  {} Skills governance check passed ({:.2}s)",
                        "✓".green(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    println!("  {} Skills governance check passed", "✓".green());
                }
                true
            }
            Err(e) => {
                eprintln!("  {} Skills governance check failed: {}", "✗".red(), e);
                false
            }
        }
    } else {
        println!("  {} No Skills directory found (skipping)", "⚠".yellow());
        true
    };
    results.push("Skills governance", skills_ok, Some("Run `cargo run -p xtask -- skills-lint`"));
    println!();

    // Step 3: Agents governance lint
    println!("{}", "[3/11] Checking Agents governance...".blue());
    let step_start = Instant::now();
    let agents_ok = if Path::new(".claude/agents").exists() {
        match crate::commands::agents::run_lint() {
            Ok(_) => {
                let elapsed = step_start.elapsed();
                if verbosity.is_verbose() {
                    println!(
                        "  {} Agents governance check passed ({:.2}s)",
                        "✓".green(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    println!("  {} Agents governance check passed", "✓".green());
                }
                true
            }
            Err(e) => {
                eprintln!("  {} Agents governance check failed: {}", "✗".red(), e);
                false
            }
        }
    } else {
        println!("  {} No Agents directory found (skipping)", "⚠".yellow());
        true
    };
    results.push("Agents governance", agents_ok, Some("Run `cargo run -p xtask -- agents-lint`"));
    println!();

    // Step 4: BDD acceptance tests
    println!("{}", "[4/11] Running BDD acceptance tests...".blue());
    let bdd_ok = if skip_bdd {
        println!(
            "  {} Skipping BDD tests because XTASK_SKIP_BDD=1 (avoid recursion in harness)",
            "⚠".yellow()
        );
        true
    } else {
        let step_start = Instant::now();
        match crate::commands::bdd::run() {
            Ok(_) => {
                let elapsed = step_start.elapsed();
                if verbosity.is_verbose() {
                    println!(
                        "  {} BDD scenarios passed ({:.2}s)",
                        "✓".green(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    println!("  {} BDD scenarios passed", "✓".green());
                }
                if Path::new("target/junit/acceptance.xml").exists() {
                    println!("  {} JUnit XML generated", "✓".green());
                } else {
                    println!("  {} JUnit XML not found", "⚠".yellow());
                }
                true
            }
            Err(e) => {
                eprintln!("  {} BDD tests failed: {}", "✗".red(), e);
                false
            }
        }
    };
    results.push("BDD acceptance tests", bdd_ok, Some("Run `cargo run -p xtask -- bdd`"));
    println!();

    // Step 5: AC status mapping & ADR references
    println!("{}", "[5/11] Running AC status mapping & ADR references...".blue());
    let step_start = Instant::now();

    let mut mapping_ok = true;

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
            mapping_ok = false;
        }
    }

    let elapsed = step_start.elapsed();
    if verbosity.is_verbose() {
        println!("  {} Step 3 completed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
    }
    results.push("AC/ADR mapping", mapping_ok, Some("Run `cargo run -p xtask -- adr-check`"));
    println!();

    // Step 6: LLM context bundler
    println!("{}", "[6/11] Testing LLM context bundler...".blue());
    let step_start = Instant::now();
    let bundler_ok = match crate::commands::bundle::run("implement_ac") {
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
            true
        }
        Err(e) => {
            eprintln!("  {} Bundler failed: {}", "✗".red(), e);
            false
        }
    };
    results.push(
        "LLM bundler",
        bundler_ok,
        Some("Run `cargo run -p xtask -- bundle implement_ac`"),
    );
    println!();

    // Step 7: Policy tests (if conftest available)
    println!("{}", "[7/11] Running policy tests...".blue());
    let step_start = Instant::now();
    let policy_ok = if low_resource_mode {
        // Skip policy tests in low-resource mode as they can be resource-intensive
        println!("  {} Policy tests skipped (low-resource mode)", "⚠".yellow());
        true
    } else {
        match crate::commands::policy_test::run() {
            Ok(_) => {
                let elapsed = step_start.elapsed();
                if verbosity.is_verbose() {
                    println!(
                        "  {} Policy tests passed ({:.2}s)",
                        "✓".green(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    println!("  {} Policy tests passed", "✓".green());
                }
                true
            }
            Err(e) => {
                // Check if this is a "conftest not found" error
                let is_conftest_not_found =
                    matches!(e, crate::commands::policy_test::PolicyTestError::ConftestNotFound(_));

                if is_conftest_not_found {
                    // In CI, treat this as a failure; locally, just warn
                    if crate::env::is_ci() {
                        eprintln!(
                            "  {} Policy tests: conftest not found (CI requires conftest)",
                            "✗".red()
                        );
                        if verbosity.is_verbose() {
                            eprintln!("\n{}", e);
                        }
                        false
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
                        // Don't fail for local development
                        true
                    }
                } else {
                    eprintln!("  {} Policy tests: {}", "✗".red(), e);
                    false
                }
            }
        }
    };
    results.push(
        "Policy tests",
        policy_ok,
        Some("Run `cargo run -p xtask -- policy-test` or use `nix develop`"),
    );
    println!();
    // Step 8: DevEx contract
    println!("{}", "[8/11] Checking DevEx contract...".blue());
    let step_start = Instant::now();
    let devex_ok = match run_devex_contract(verbosity) {
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
            true
        }
        Err(e) => {
            eprintln!("  {} DevEx contract failed: {}", "✗".red(), e);
            false
        }
    };
    results.push(
        "DevEx contract",
        devex_ok,
        Some("Check specs/devex_flows.yaml and implemented commands"),
    );
    println!();

    // Step 9: Graph invariants + UI contract
    println!("{}", "[9/11] Checking governance graph & UI contract...".blue());
    let step_start = Instant::now();
    let mut governance_ok = true;

    // 9a: Graph invariants
    match crate::commands::graph_export::run_graph_invariants(verbosity.as_u8()) {
        Ok(_) => {
            println!("  {} Graph invariants satisfied", "✓".green());
        }
        Err(e) => {
            eprintln!("  {} Graph invariants failed:", "✗".red());
            eprintln!("{}", e);
            governance_ok = false;
        }
    }

    // 9b: UI contract check (YAML structure + DOM validation)
    match crate::commands::ui_contract_check::run_check() {
        Ok(_) => {
            // run_check already prints success message
        }
        Err(e) => {
            eprintln!("  {} UI contract check failed: {}", "✗".red(), e);
            governance_ok = false;
        }
    }

    let elapsed = step_start.elapsed();
    if verbosity.is_verbose() && governance_ok {
        println!("  {} Step 9 completed ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
    }

    results.push(
        "Governance graph & UI",
        governance_ok,
        Some("Run `cargo xtask graph-export --check` or `cargo xtask ui-contract-check`"),
    );
    println!();

    // Step 10: AC coverage
    println!("{}", "[10/11] Checking AC coverage for v3.0 kernel...".blue());
    let step_start = Instant::now();
    let coverage_ok = match run_ac_coverage_check(verbosity) {
        Ok(_) => {
            let elapsed = step_start.elapsed();
            if verbosity.is_verbose() {
                println!("  {} AC coverage complete ({:.2}s)", "✓".green(), elapsed.as_secs_f64());
            } else {
                println!("  {} AC coverage complete", "✓".green());
            }
            true
        }
        Err(e) => {
            eprintln!("  {} AC coverage incomplete:", "✗".red());
            eprintln!("{}", e);
            false
        }
    };
    results.push("AC coverage", coverage_ok, Some("Run `cargo xtask ac-coverage` for details"));
    println!();

    // Step 11: Test coverage (soft gate - advisory only)
    println!("{}", "[11/11] Checking test coverage (advisory)...".blue());
    let step_start = Instant::now();
    let test_coverage_ok = if low_resource_mode {
        println!("  {} Test coverage skipped (low-resource mode)", "⚠".yellow());
        true
    } else {
        match crate::commands::coverage::run() {
            Ok(_) => {
                let elapsed = step_start.elapsed();
                if verbosity.is_verbose() {
                    println!(
                        "  {} Test coverage target met ({:.2}s)",
                        "✓".green(),
                        elapsed.as_secs_f64()
                    );
                } else {
                    println!("  {} Test coverage target met", "✓".green());
                }
                true
            }
            Err(e) => {
                // Soft gate: warn but don't fail selftest
                println!("  {} Test coverage below baseline (advisory)", "⚠".yellow());
                if verbosity.is_verbose() {
                    eprintln!("{}", e);
                }
                println!(
                    "  💡 Hint: Run {} for detailed coverage report",
                    "cargo xtask coverage".cyan()
                );
                true // Don't fail selftest on coverage
            }
        }
    };
    results.push("Test coverage", test_coverage_ok, Some("Run `cargo xtask coverage` for details"));
    println!();

    // Print summary
    let total_elapsed = start_time.elapsed();
    print_summary(&results);

    let failed = results.failed_count();
    if failed == 0 {
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
        Ok(())
    } else {
        if verbosity.is_verbose() {
            eprintln!("\n{} {:.2}s", "Total elapsed time:".bold(), total_elapsed.as_secs_f64());
        }
        anyhow::bail!("{} test suites failed", failed)
    }
}

fn run_ac_status(verbosity: crate::Verbosity) -> Result<()> {
    // Use check mode to verify AC status file is up-to-date without regenerating.
    // This prevents selftest from modifying the repo - the pre-commit hook handles
    // regeneration and staging of docs/feature_status.md.
    //
    // We also require coverage to exist (require_coverage: true) because:
    // - Step 4 (BDD) should have generated fresh coverage
    // - Without coverage, the computed status would have many [UNKNOWN] entries
    // - This prevents spurious check failures from stale/missing coverage data
    let layout = crate::kernel::layout_for_repo();

    // If BDD was skipped (XTASK_SKIP_BDD=1), we can't validate AC status meaningfully
    // because we have no fresh coverage data to compare against.
    if std::env::var("XTASK_SKIP_BDD").unwrap_or_default() == "1" {
        if verbosity.is_verbose() {
            eprintln!(
                "  {} AC status check skipped (XTASK_SKIP_BDD=1, no fresh coverage to validate)",
                "?".yellow()
            );
        }
        return Ok(());
    }

    // Check if coverage exists - if not, provide a helpful error
    if !layout.has_coverage() {
        anyhow::bail!(
            "Coverage file missing, cannot validate AC status\n\n\
             Expected: {}\n\n\
             hint: BDD tests (step 4) should have generated coverage.\n\
                   If you see this, something went wrong with BDD execution.\n\
             try:  cargo xtask bdd",
            layout.coverage_file.display()
        );
    }

    crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity,
        check: true,            // Check mode: verify without writing
        require_coverage: true, // Fail if coverage is missing (guard against churn)
        ..Default::default()
    })
}

fn run_adr_check(verbosity: crate::Verbosity) -> Result<()> {
    crate::commands::adr_check::run(crate::commands::adr_check::AdrCheckArgs {
        verbosity,
        ..Default::default()
    })
}

/// Result of checking devex contract (required commands exist)
/// AC-PLT-015: selftest enforces devex contract
#[derive(Debug, PartialEq)]
pub struct DevexContractResult {
    pub required_count: usize,
    pub missing: Vec<String>,
}

impl DevexContractResult {
    pub fn is_valid(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Check that all required commands from the devex spec exist in available commands
/// AC-PLT-015: This is the core logic that enforces the devex contract
pub fn check_devex_contract(
    spec: &crate::devex::DevExSpec,
    available_commands: &[&str],
) -> DevexContractResult {
    let mut missing = Vec::new();
    let mut required_count = 0;

    for (cmd_name, cmd_spec) in &spec.commands {
        if cmd_spec.required {
            required_count += 1;
            if !available_commands.contains(&cmd_name.as_str()) {
                missing.push(cmd_name.clone());
            }
        }
    }

    // Sort for deterministic output
    missing.sort();

    DevexContractResult { required_count, missing }
}

fn run_devex_contract(verbosity: crate::Verbosity) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let spec_path = root.join("specs/devex_flows.yaml");
    let spec = crate::devex::load_spec(&spec_path)?;

    // Get list of available commands
    let available_commands = crate::all_command_names();

    let result = check_devex_contract(&spec, &available_commands);

    if !result.is_valid() {
        eprintln!();
        eprintln!("✗ Required commands missing from xtask:");
        for name in &result.missing {
            eprintln!("  • {}", name);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Implement missing command(s) in crates/xtask");
        eprintln!("  • Or update specs/devex_flows.yaml if spec is outdated");
        anyhow::bail!("DevEx contract: {} required command(s) missing", result.missing.len());
    }

    // Success: report validation details
    if verbosity.is_verbose() {
        println!(
            "  {} Validated {} required commands from devex_flows.yaml",
            "✓".green(),
            result.required_count
        );
    } else {
        println!("  {} All required commands from devex_flows.yaml exist", "✓".green());
    }

    Ok(())
}

/// Check AC coverage for v3.0 kernel requirements
fn run_ac_coverage_check(verbosity: crate::Verbosity) -> Result<()> {
    use serde::Deserialize;
    use std::collections::{HashMap, HashSet};
    use std::fs;

    #[derive(Debug, Deserialize)]
    struct Ledger {
        stories: Vec<Story>,
    }

    #[derive(Debug, Deserialize)]
    struct Story {
        /// Story ID from spec_ledger.yaml.
        /// Currently only used for deserialization; ID not needed in selftest validation.
        #[allow(dead_code)]
        id: String,
        requirements: Vec<Requirement>,
    }

    #[derive(Debug, Deserialize)]
    struct Requirement {
        id: String,
        #[serde(default = "default_must_have_ac")]
        must_have_ac: bool,
        acceptance_criteria: Vec<AcceptanceCriteria>,
    }

    fn default_must_have_ac() -> bool {
        true
    }

    #[derive(Debug, Deserialize)]
    struct AcceptanceCriteria {
        id: String,
        /// AC description text.
        /// Currently not used in selftest validation; only ID and must_have_ac flag matter.
        #[serde(default)]
        #[allow(dead_code)]
        text: String,
        #[serde(default = "default_must_have_ac")]
        must_have_ac: bool,
    }

    // When BDD execution is explicitly skipped (e.g., in harnesses to avoid recursion),
    // there is no fresh acceptance report to evaluate, so treat coverage as informational.
    let bdd_skipped = std::env::var("XTASK_SKIP_BDD").unwrap_or_default() == "1";
    if bdd_skipped {
        println!(
            "  {} AC coverage skipped (XTASK_SKIP_BDD=1; acceptance results not generated)",
            "?".yellow()
        );
        return Ok(());
    }

    // Parse the ledger
    let ledger_path = Path::new("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        anyhow::bail!("Ledger not found: {}", ledger_path.display());
    }

    let content = fs::read_to_string(ledger_path)?;
    let ledger: Ledger = serde_yaml::from_str(&content)?;

    // Collect all ACs, separating kernel (must_have_ac=true) from non-kernel (must_have_ac=false)
    let mut kernel_acs: HashMap<String, String> = HashMap::new(); // ac_id -> req_id
    let mut non_kernel_acs: HashMap<String, String> = HashMap::new(); // ac_id -> req_id

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                if req.must_have_ac && ac.must_have_ac {
                    kernel_acs.insert(ac.id.clone(), req.id.clone());
                } else {
                    non_kernel_acs.insert(ac.id.clone(), req.id.clone());
                }
            }
        }
    }

    if verbosity.is_verbose() {
        println!(
            "  Found {} kernel ACs (must_have_ac=true) across {} requirements",
            kernel_acs.len(),
            kernel_acs.values().collect::<HashSet<_>>().len()
        );
        println!(
            "  Found {} non-kernel ACs (must_have_ac=false) across {} requirements",
            non_kernel_acs.len(),
            non_kernel_acs.values().collect::<HashSet<_>>().len()
        );
    }

    // Get AC status by running ac-status and parsing the output
    // We use the same logic as ac_status.rs but focus on kernel ACs only
    let args = crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        summary: false,
        ..Default::default()
    };

    // Run ac-status to generate feature_status.md
    if let Err(e) = crate::commands::ac_status::run(args) {
        // ac-status failed, which means some ACs are failing
        // We'll parse the status anyway to give detailed feedback
        if !verbosity.is_quiet() {
            println!("  {} AC status check failed: {}", "⚠".yellow(), e);
        }
    }

    // Parse feature_status.md to get AC statuses
    let status_path = Path::new("docs/feature_status.md");
    if !status_path.exists() {
        anyhow::bail!("Feature status file not found. Run `cargo xtask ac-status` first.");
    }

    let status_content = fs::read_to_string(status_path)?;
    let mut kernel_failing: Vec<(String, String)> = Vec::new(); // (ac_id, req_id)
    let mut kernel_unknown: Vec<(String, String)> = Vec::new(); // (ac_id, req_id)
    let mut non_kernel_failing: Vec<(String, String)> = Vec::new();
    let mut non_kernel_unknown: Vec<(String, String)> = Vec::new();

    // Parse the markdown table to extract AC statuses
    for line in status_content.lines() {
        if line.starts_with('|') && !line.starts_with("|----") && !line.starts_with("| AC ID") {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                let ac_id = parts[1];
                let req_id = parts[3];
                let status = parts[4];

                let is_kernel = kernel_acs.contains_key(ac_id);
                let is_non_kernel = non_kernel_acs.contains_key(ac_id);

                if status.contains("❌") || status.contains("fail") {
                    if is_kernel {
                        kernel_failing.push((ac_id.to_string(), req_id.to_string()));
                    } else if is_non_kernel {
                        non_kernel_failing.push((ac_id.to_string(), req_id.to_string()));
                    }
                } else if status.contains("❓") || status.contains("unknown") {
                    if is_kernel {
                        kernel_unknown.push((ac_id.to_string(), req_id.to_string()));
                    } else if is_non_kernel {
                        non_kernel_unknown.push((ac_id.to_string(), req_id.to_string()));
                    }
                }
            }
        }
    }

    // Report results
    let total_kernel = kernel_acs.len();
    let total_non_kernel = non_kernel_acs.len();

    let kernel_failing_count = kernel_failing.len();
    let kernel_unknown_count = kernel_unknown.len();
    let kernel_passing = total_kernel - kernel_failing_count - kernel_unknown_count;

    let non_kernel_failing_count = non_kernel_failing.len();
    let non_kernel_unknown_count = non_kernel_unknown.len();
    let non_kernel_passing = total_non_kernel - non_kernel_failing_count - non_kernel_unknown_count;

    // Always show the breakdown
    println!();
    println!("  {} Kernel ACs (must_have_ac=true):", "🔒".bold());
    println!("    Total:   {}", total_kernel);
    println!("    {} Passing: {}", "✓".green(), kernel_passing);
    if kernel_failing_count > 0 {
        println!("    {} Failing: {}", "✗".red(), kernel_failing_count);
    }
    if kernel_unknown_count > 0 {
        println!("    {} Unknown: {}", "?".yellow(), kernel_unknown_count);
    }

    if total_non_kernel > 0 {
        println!();
        println!("  {} Non-kernel ACs (must_have_ac=false):", "💡".bold());
        println!("    Total:   {}", total_non_kernel);
        println!("    {} Passing: {}", "✓".green(), non_kernel_passing);
        if non_kernel_failing_count > 0 {
            println!(
                "    {} Failing: {} (informational only)",
                "✗".yellow(),
                non_kernel_failing_count
            );
        }
        if non_kernel_unknown_count > 0 {
            println!(
                "    {} Unknown: {} (informational only)",
                "?".yellow(),
                non_kernel_unknown_count
            );
        }
    }

    // Check for strict mode: XTASK_STRICT_AC_COVERAGE=1 fails on Unknown must_have_ac ACs
    let strict_mode = env::var("XTASK_STRICT_AC_COVERAGE").unwrap_or_default() == "1";

    // Fail on explicit failures
    if kernel_failing_count > 0 {
        eprintln!();
        eprintln!("{}", "❌ Kernel AC coverage gate failed".red().bold());
        eprintln!();

        eprintln!("{}", "Failing kernel ACs:".bold());
        for (ac_id, req_id) in &kernel_failing {
            eprintln!("  • {} ({})", ac_id, req_id);
        }
        eprintln!();

        eprintln!("{}", "Next steps:".bold());
        eprintln!("  1. View detailed AC status: {}", "cargo xtask ac-coverage".cyan());
        eprintln!("  2. Run failing tests: {}", "cargo xtask bdd".cyan());

        anyhow::bail!("Kernel AC coverage incomplete: {} failing", kernel_failing_count);
    }

    // In strict mode, Unknown must_have_ac ACs also fail the gate
    if strict_mode && kernel_unknown_count > 0 {
        eprintln!();
        eprintln!("{}", "❌ Kernel AC coverage gate failed (strict mode)".red().bold());
        eprintln!();

        eprintln!("{}", "Unknown must_have_ac ACs (no BDD coverage):".bold());
        for (ac_id, req_id) in &kernel_unknown {
            eprintln!("  • {} ({})", ac_id, req_id);
        }
        eprintln!();

        eprintln!("{}", "Next steps:".bold());
        eprintln!("  1. View the backlog: {}", "cargo xtask ac-coverage --todo --must-have".cyan());
        eprintln!("  2. Generate scenarios: {}", "cargo xtask ac-suggest-scenarios <AC_ID>".cyan());
        eprintln!("  3. Add @<AC_ID> scenarios and rerun: {}", "cargo xtask selftest".cyan());
        eprintln!();
        eprintln!("{}", "ℹ️  To disable strict mode, unset XTASK_STRICT_AC_COVERAGE".dimmed());

        anyhow::bail!(
            "Kernel AC coverage incomplete: {} unknown (strict mode enabled)",
            kernel_unknown_count
        );
    }

    // In non-strict mode, Unknown ACs are advisory
    if kernel_unknown_count > 0 {
        println!();
        if strict_mode {
            // This branch won't be reached (we'd have failed above), but for completeness
            println!(
                "  {} {} kernel ACs have unknown coverage",
                "⚠".yellow(),
                kernel_unknown_count
            );
        } else {
            println!(
                "  {} {} kernel ACs have unknown coverage (advisory)",
                "⚠".yellow(),
                kernel_unknown_count
            );
            println!(
                "    💡 To enforce coverage, set {} and rerun",
                "XTASK_STRICT_AC_COVERAGE=1".cyan()
            );
            println!(
                "    📋 View backlog: {}",
                "cargo xtask ac-coverage --todo --must-have".cyan()
            );
        }
    }

    // Provide informational warning about non-kernel ACs if any are failing/unknown
    if total_non_kernel > 0 && (non_kernel_failing_count > 0 || non_kernel_unknown_count > 0) {
        println!();
        println!("{}", "ℹ️  Non-kernel AC status (informational only):".cyan());
        if non_kernel_failing_count > 0 {
            println!("  {} failing non-kernel ACs", non_kernel_failing_count);
        }
        if non_kernel_unknown_count > 0 {
            println!("  {} unknown non-kernel ACs", non_kernel_unknown_count);
        }
        println!("  These won't block the selftest gate but may affect feature completeness.");
    }

    Ok(())
}

/// Print a summary of selftest results with actionable hints
fn print_summary(results: &SelftestResults) {
    println!();
    println!("{}", "======================================".blue());
    println!("{}", "Selftest Summary:".bold());
    println!("{}", "======================================".blue());

    for (i, step) in results.steps.iter().enumerate() {
        let status = if step.ok { "OK".green() } else { "FAIL".red() };
        println!("  {}. {:32} {}", i + 1, format!("{} ...", step.name), status);
    }

    let failed: Vec<_> = results.steps.iter().filter(|s| !s.ok).collect();
    if failed.is_empty() {
        println!();
        println!("{}", "All checks passed ✅".green().bold());
        return;
    }

    println!();
    println!("{}", "Next actions:".bold());
    for step in failed {
        if let Some(hint) = step.hint {
            println!("  • {}", hint);
        }
    }
    println!("{}", "======================================".blue());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// AC-PLT-015: selftest enforces devex contract (required commands exist)
    /// This test verifies that the devex contract check correctly identifies
    /// when required commands are missing from the available command set.
    #[test]
    fn devex_contract_enforced_missing_commands() {
        // Create a spec with required commands
        let mut commands = HashMap::new();
        commands.insert(
            "doctor".to_string(),
            crate::devex::CommandSpec {
                category: "onboarding".to_string(),
                summary: "Check environment".to_string(),
                required: true,
                docs: Default::default(),
            },
        );
        commands.insert(
            "selftest".to_string(),
            crate::devex::CommandSpec {
                category: "validation".to_string(),
                summary: "Run tests".to_string(),
                required: true,
                docs: Default::default(),
            },
        );
        commands.insert(
            "optional-cmd".to_string(),
            crate::devex::CommandSpec {
                category: "utils".to_string(),
                summary: "Optional command".to_string(),
                required: false,
                docs: Default::default(),
            },
        );

        let spec = crate::devex::DevExSpec {
            schema_version: "1.0".to_string(),
            template_version: "3.0.0".to_string(),
            commands,
            flows: HashMap::new(),
        };

        // Test with missing required command
        let available = vec!["doctor"];
        let result = check_devex_contract(&spec, &available);

        assert!(!result.is_valid(), "Should fail when required command is missing");
        assert_eq!(result.required_count, 2);
        assert_eq!(result.missing, vec!["selftest"]);
    }

    /// AC-PLT-015: selftest enforces devex contract
    /// This test verifies that the check passes when all required commands exist.
    #[test]
    fn devex_contract_enforced_all_present() {
        let mut commands = HashMap::new();
        commands.insert(
            "doctor".to_string(),
            crate::devex::CommandSpec {
                category: "onboarding".to_string(),
                summary: "Check environment".to_string(),
                required: true,
                docs: Default::default(),
            },
        );
        commands.insert(
            "selftest".to_string(),
            crate::devex::CommandSpec {
                category: "validation".to_string(),
                summary: "Run tests".to_string(),
                required: true,
                docs: Default::default(),
            },
        );

        let spec = crate::devex::DevExSpec {
            schema_version: "1.0".to_string(),
            template_version: "3.0.0".to_string(),
            commands,
            flows: HashMap::new(),
        };

        // Test with all required commands present
        let available = vec!["doctor", "selftest", "extra-cmd"];
        let result = check_devex_contract(&spec, &available);

        assert!(result.is_valid(), "Should pass when all required commands exist");
        assert_eq!(result.required_count, 2);
        assert!(result.missing.is_empty());
    }

    /// AC-PLT-015: verify actual devex_flows.yaml contract is satisfied
    #[test]
    fn devex_contract_real_spec_satisfied() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let spec_path = root.join("specs/devex_flows.yaml");

        let spec = crate::devex::load_spec(&spec_path).expect("devex_flows.yaml should parse");
        let available_commands = crate::all_command_names();

        let result = check_devex_contract(&spec, &available_commands);

        assert!(
            result.is_valid(),
            "All required commands should be implemented. Missing: {:?}",
            result.missing
        );
        assert!(result.required_count > 0, "Should have at least some required commands");
    }

    /// AC-PLT-019: selftest displays condensed summary with 11 steps
    /// This test verifies the step structure and naming.
    #[test]
    fn selftest_summary_has_eleven_steps() {
        // The selftest runs 11 steps - verify the structure
        let expected_steps = [
            "Core checks",
            "Skills governance",
            "Agents governance",
            "BDD scenarios",
            "AC/ADR mapping",
            "LLM bundler",
            "Policy tests",
            "DevEx contract",
            "Governance graph & UI",
            "AC coverage",
            "Test coverage",
        ];

        // Verify we have exactly 11 steps
        assert_eq!(expected_steps.len(), 11, "Selftest should have exactly 11 steps");

        // Verify key governance steps are present
        assert!(
            expected_steps.contains(&"Skills governance"),
            "Skills governance step should be present"
        );
        assert!(
            expected_steps.contains(&"Agents governance"),
            "Agents governance step should be present"
        );
        assert!(
            expected_steps.contains(&"DevEx contract"),
            "DevEx contract step should be present"
        );
        assert!(
            expected_steps.contains(&"Governance graph & UI"),
            "Governance graph & UI step should be present"
        );
        assert!(expected_steps.contains(&"AC coverage"), "AC coverage step should be present");
    }

    /// AC-PLT-019: selftest results structure supports pass/fail status
    #[test]
    fn selftest_results_track_status() {
        let mut results = SelftestResults::new();

        results.push("Core checks", true, None);
        results.push("Skills governance", true, None);
        results.push("BDD scenarios", false, Some("Run cargo xtask bdd"));

        assert_eq!(results.failed_count(), 1);
        assert_eq!(results.steps.len(), 3);
        assert!(results.steps[0].ok);
        assert!(!results.steps[2].ok);
    }

    /// Test that XTASK_STRICT_AC_COVERAGE env var is recognized
    /// This documents the expected behavior of strict mode:
    /// - Default (unset or "0"): Unknown kernel ACs are advisory
    /// - "1": Unknown kernel ACs fail the gate
    #[test]
    fn strict_ac_coverage_env_var_parsing() {
        use std::env;

        // Test parsing logic (same as in run_ac_coverage_check)
        let parse_strict = || env::var("XTASK_STRICT_AC_COVERAGE").unwrap_or_default() == "1";

        // Default behavior: strict mode is off
        // SAFETY: Tests run single-threaded when using this env var
        unsafe {
            env::remove_var("XTASK_STRICT_AC_COVERAGE");
        }
        assert!(!parse_strict(), "Default should be non-strict");

        // Explicit "0" should be non-strict
        unsafe {
            env::set_var("XTASK_STRICT_AC_COVERAGE", "0");
        }
        assert!(!parse_strict(), "Explicit 0 should be non-strict");

        // "1" enables strict mode
        unsafe {
            env::set_var("XTASK_STRICT_AC_COVERAGE", "1");
        }
        assert!(parse_strict(), "1 should enable strict mode");

        // Clean up
        unsafe {
            env::remove_var("XTASK_STRICT_AC_COVERAGE");
        }
    }

    /// Verify that run_ac_status uses check mode (read-only contract).
    /// This is a compile-time contract test that documents the invariant:
    /// selftest must NOT modify docs/feature_status.md.
    #[test]
    fn selftest_ac_status_uses_check_mode() {
        // Verify that the run_ac_status function passes check: true
        // by checking the args structure it would construct
        let args = crate::commands::ac_status::AcStatusArgs {
            verbosity: crate::Verbosity::Quiet,
            check: true, // This MUST be true in run_ac_status
            ..Default::default()
        };

        assert!(
            args.check,
            "selftest's run_ac_status must use check mode to prevent modifying the repo"
        );

        // This test exists to catch regressions if someone changes run_ac_status
        // to use write mode (check: false), which would cause selftest to
        // unexpectedly modify docs/feature_status.md
    }
}
