use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    println!("{}", "📚 Checking documentation consistency...".blue().bold());
    println!();

    let mut issues = 0;

    // Check version alignment
    print!("Version alignment... ");
    match check_version_alignment() {
        Ok(_) => println!("{}", "✓ Consistent".green()),
        Err(e) => {
            println!("{}", "✗ Mismatch".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    // Check ADR references
    print!("ADR references... ");
    match crate::commands::adr_check::run(crate::commands::adr_check::AdrCheckArgs {
        verbosity: crate::Verbosity::Quiet,
        ..Default::default()
    }) {
        Ok(_) => println!("{}", "✓ Valid".green()),
        Err(e) => {
            println!("{}", "✗ Issues found".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    // Check AC status cleanliness
    print!("AC status consistency... ");
    match check_ac_status_clean() {
        Ok(_) => println!("{}", "✓ Up to date".green()),
        Err(e) => {
            println!("{}", "✗ Out of sync".red());
            eprintln!("  {}", e);
            issues += 1;
        }
    }

    println!();
    if issues == 0 {
        println!("{} Documentation is consistent", "✓".green().bold());
    } else {
        println!("{} {} issue(s) found", "✗".red().bold(), issues);
        println!();
        println!("{}", "To fix:".bold());
        println!("  • Align versions: {}", "cargo xtask release-prepare X.Y.Z".cyan());
        println!("  • Or manually sync: {}", "README.md, CLAUDE.md, spec_ledger.yaml".dimmed());
        println!("  • Commit generated docs if out of sync");
        println!("  • See: {}", "docs/RELEASE_PLAYBOOK.md".dimmed());
    }

    if issues > 0 {
        anyhow::bail!("{} documentation issues", issues);
    }

    Ok(())
}

fn check_version_alignment() -> Result<()> {
    // Extract versions from key files
    let readme_version = extract_version_from_readme()?;
    let ledger_version = extract_version_from_ledger()?;
    let claude_version = extract_version_from_claude()?;

    if readme_version != ledger_version || readme_version != claude_version {
        anyhow::bail!(
            "Version mismatch: README={}, ledger={}, CLAUDE={}",
            readme_version,
            ledger_version,
            claude_version
        );
    }

    Ok(())
}

fn extract_version_from_readme() -> Result<String> {
    let content = fs::read_to_string("README.md")?;
    // Look for "**Current Template Version:** vX.Y.Z"
    for line in content.lines() {
        if line.contains("Current Template Version") {
            if let Some(version) = line.split('v').nth(1) {
                return Ok(version.split_whitespace().next().unwrap_or("unknown").to_string());
            }
        }
    }
    Ok("unknown".to_string())
}

fn extract_version_from_ledger() -> Result<String> {
    let content = fs::read_to_string("specs/spec_ledger.yaml")?;
    for line in content.lines() {
        if line.trim().starts_with("template_version:") {
            if let Some(version) = line.split(':').nth(1) {
                return Ok(version.trim().trim_matches('"').to_string());
            }
        }
    }
    Ok("unknown".to_string())
}

fn extract_version_from_claude() -> Result<String> {
    let content = fs::read_to_string("CLAUDE.md")?;
    // Look for "**Template Version:** vX.Y.Z"
    for line in content.lines() {
        if line.contains("Template Version") {
            if let Some(version) = line.split('v').nth(1) {
                return Ok(version.split_whitespace().next().unwrap_or("unknown").to_string());
            }
        }
    }
    Ok("unknown".to_string())
}

fn check_ac_status_clean() -> Result<()> {
    // Run ac-status to regenerate
    crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        ..Default::default()
    })?;

    // Check if git reports changes
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "docs/feature_status.md"])
        .output()?;

    let status = String::from_utf8_lossy(&output.stdout);
    if !status.trim().is_empty() {
        anyhow::bail!(
            "docs/feature_status.md is out of date. Run 'cargo xtask ac-status' and commit."
        );
    }

    Ok(())
}
