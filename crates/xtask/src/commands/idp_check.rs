use anyhow::Result;
use colored::Colorize;
use std::process::Command;

/// Run IDP integration checks (OpenAPI lint + Backstage plugin + TS config validation)
///
/// This is a non-kernel convenience command that validates the IDP consumer
/// surface is healthy. It does NOT gate selftest - it's for integration
/// ergonomics when bumping dependencies or changing contracts.
pub fn run() -> Result<()> {
    println!("{}", "🔌 Validating IDP surface...".blue().bold());
    println!();

    let mut results: Vec<(&str, CheckResult)> = Vec::new();

    // Step 1: OpenAPI lint
    print!("  {} OpenAPI lint ", "[1/3]".dimmed());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let openapi_result = match run_openapi_lint() {
        Ok(_) => CheckResult::Pass,
        Err(e) => CheckResult::Fail(e.to_string()),
    };
    print_result(&openapi_result);
    results.push(("OpenAPI lint", openapi_result));

    // Step 2: Backstage plugin checks
    print!("  {} Backstage plugin ", "[2/3]".dimmed());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let plugin_dir = std::path::Path::new("examples/backstage-plugin");
    let plugin_result = if !plugin_dir.exists() {
        CheckResult::Skip("examples/backstage-plugin not present".to_string())
    } else {
        match run_plugin_checks(plugin_dir) {
            Ok(_) => CheckResult::Pass,
            Err(e) => CheckResult::Fail(e.to_string()),
        }
    };
    print_result(&plugin_result);
    results.push(("Backstage plugin", plugin_result));

    // Step 3: TypeScript config validation
    print!("  {} TS config ", "[3/3]".dimmed());
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let ts_result = run_ts_config_check();
    print_result(&ts_result);
    results.push(("TS config", ts_result));

    println!();

    // Summary
    let failures: Vec<_> =
        results.iter().filter(|(_, r)| matches!(r, CheckResult::Fail(_))).collect();
    let skipped: Vec<_> =
        results.iter().filter(|(_, r)| matches!(r, CheckResult::Skip(_))).collect();

    if !skipped.is_empty() {
        for (name, result) in &skipped {
            if let CheckResult::Skip(reason) = result {
                println!("  {} {} skipped ({})", "⚠".yellow(), name, reason.dimmed());
                // Provide friendly guidance for common skip scenarios
                if *name == "Backstage plugin" && reason.contains("not present") {
                    println!(
                        "    {}",
                        "This is fine if you're not using the reference Backstage plugin yet."
                            .dimmed()
                    );
                }
            }
        }
    }

    if failures.is_empty() {
        println!("{}", "IDP surface validation PASSED".green().bold());
        Ok(())
    } else {
        println!("{}", "IDP surface validation FAILED".red().bold());
        println!();
        for (name, result) in &failures {
            if let CheckResult::Fail(reason) = result {
                println!("  {} {}: {}", "✗".red(), name, reason);
            }
        }
        println!();
        println!(
            "{}",
            "Fix the contract (Rust/OpenAPI) first, then update the TypeScript client.".dimmed()
        );
        anyhow::bail!("IDP integration checks failed")
    }
}

enum CheckResult {
    Pass,
    Fail(String),
    Skip(String),
}

fn print_result(result: &CheckResult) {
    match result {
        CheckResult::Pass => println!("{}", "✓".green().bold()),
        CheckResult::Fail(_) => println!("{}", "✗".red().bold()),
        CheckResult::Skip(_) => println!("{}", "⊘".yellow()),
    }
}

/// Run OpenAPI linting with Redocly CLI
fn run_openapi_lint() -> Result<()> {
    let openapi_path = std::path::Path::new("specs/openapi/openapi.yaml");
    if !openapi_path.exists() {
        println!("  {} OpenAPI spec not found at {}", "⚠".yellow(), openapi_path.display());
        return Ok(());
    }

    println!("  Running: npx @redocly/cli lint specs/openapi/openapi.yaml");

    let output = Command::new("npx")
        .args([
            "-y",
            "@redocly/cli@1.21.0",
            "lint",
            "--config",
            "redocly.yaml",
            "specs/openapi/openapi.yaml",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        anyhow::bail!("Redocly lint failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    Ok(())
}

/// Run Backstage plugin type checks and tests
fn run_plugin_checks(plugin_dir: &std::path::Path) -> Result<()> {
    // Check if pnpm is available
    if which::which("pnpm").is_err() {
        anyhow::bail!("pnpm not found");
    }

    // Run pnpm check (type checking)
    let check_output = Command::new("pnpm")
        .arg("run")
        .arg("check")
        .env("CI", "1")
        .current_dir(plugin_dir)
        .output()?;

    if !check_output.status.success() {
        anyhow::bail!("TypeScript type check failed");
    }

    // Run PlatformClient tests
    let test_output = Command::new("pnpm")
        .args([
            "exec",
            "backstage-cli",
            "package",
            "test",
            "--passWithNoTests",
            "--testPathPattern=PlatformClient.test.ts",
        ])
        .env("CI", "1")
        .current_dir(plugin_dir)
        .output()?;

    if !test_output.status.success() {
        anyhow::bail!("PlatformClient tests failed");
    }

    Ok(())
}

/// Run TypeScript config validation
fn run_ts_config_check() -> CheckResult {
    match super::validate_ts_config::validate_workspace(std::path::Path::new("."), false) {
        Ok(summary) if summary.files_checked == 0 => {
            CheckResult::Skip("no tsconfig.json files found".to_string())
        }
        Ok(summary) if summary.violations == 0 => CheckResult::Pass,
        Ok(summary) => CheckResult::Fail(format!(
            "{} TypeScript config violation(s) found",
            summary.violations
        )),
        Err(e) => CheckResult::Fail(format!("TypeScript config validation failed: {e}")),
    }
}
