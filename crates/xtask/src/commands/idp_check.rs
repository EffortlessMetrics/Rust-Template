use anyhow::Result;
use colored::Colorize;
use std::process::Command;

/// Run IDP integration checks (OpenAPI lint + Backstage plugin tests)
///
/// This is a non-kernel convenience command that validates the IDP consumer
/// surface is healthy. It does NOT gate selftest - it's for integration
/// ergonomics when bumping dependencies or changing contracts.
pub fn run() -> Result<()> {
    println!("{}", "🔌 Running IDP integration checks...".blue().bold());
    println!();
    println!("{}", "This validates the IDP consumer surface:".dimmed());
    println!("  1. {} - Validate OpenAPI schema", "OpenAPI lint (redocly)".cyan());
    println!("  2. {} - TypeScript types and API tests", "Backstage plugin checks".cyan());
    println!();

    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    // Step 1: OpenAPI lint
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "Step 1: OpenAPI Schema Validation".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    match run_openapi_lint() {
        Ok(_) => {
            println!("{} OpenAPI schema is valid", "✓".green().bold());
        }
        Err(e) => {
            println!("{} OpenAPI lint failed: {}", "✗".red().bold(), e);
            failures.push(format!("openapi-lint: {}", e));
        }
    }

    println!();

    // Step 2: Backstage plugin checks
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "Step 2: Backstage Plugin Checks".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let plugin_dir = std::path::Path::new("examples/backstage-plugin");
    if !plugin_dir.exists() {
        println!("{} Backstage plugin not found at examples/backstage-plugin", "⚠".yellow().bold());
        warnings.push("backstage-plugin directory not found".to_string());
    } else {
        match run_plugin_checks(plugin_dir) {
            Ok(_) => {
                println!("{} Backstage plugin checks passed", "✓".green().bold());
            }
            Err(e) => {
                println!("{} Plugin checks failed: {}", "✗".red().bold(), e);
                failures.push(format!("backstage-plugin: {}", e));
            }
        }
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "IDP Check Summary".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    if !warnings.is_empty() {
        println!();
        println!("{} {} warning(s):", "⚠".yellow().bold(), warnings.len());
        for warning in &warnings {
            println!("  - {}", warning);
        }
    }

    if failures.is_empty() {
        println!();
        println!("{} IDP integration surface is healthy!", "✓".green().bold());
        println!();
        println!("{}", "The OpenAPI contract and Backstage plugin are aligned.".dimmed());
        Ok(())
    } else {
        println!();
        println!("{} {} check(s) failed:", "✗".red().bold(), failures.len());
        for (i, failure) in failures.iter().enumerate() {
            println!("  {}. {}", i + 1, failure);
        }
        println!();
        println!(
            "{}",
            "Fix the contract (Rust/OpenAPI) first, then update the TypeScript client.".dimmed()
        );
        anyhow::bail!("IDP integration checks failed")
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
        println!("  {} pnpm not found, skipping plugin checks", "⚠".yellow());
        return Ok(());
    }

    // Run pnpm check (type checking)
    println!("  Running: pnpm run check");
    let check_output =
        Command::new("pnpm").arg("run").arg("check").current_dir(plugin_dir).output()?;

    if !check_output.status.success() {
        let stderr = String::from_utf8_lossy(&check_output.stderr);
        let stdout = String::from_utf8_lossy(&check_output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        anyhow::bail!("TypeScript type check failed");
    }

    // Run PlatformClient tests
    println!("  Running: backstage-cli package test PlatformClient.test.ts");
    let test_output = Command::new("pnpm")
        .args([
            "exec",
            "backstage-cli",
            "package",
            "test",
            "--passWithNoTests",
            "--testPathPattern=PlatformClient.test.ts",
        ])
        .current_dir(plugin_dir)
        .output()?;

    if !test_output.status.success() {
        let stderr = String::from_utf8_lossy(&test_output.stderr);
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        anyhow::bail!("PlatformClient tests failed");
    }

    let stdout = String::from_utf8_lossy(&test_output.stdout);
    // Only show last few lines (test summary)
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() > 5 {
        for line in &lines[lines.len() - 5..] {
            println!("  {}", line);
        }
    } else {
        for line in &lines {
            println!("  {}", line);
        }
    }

    Ok(())
}
