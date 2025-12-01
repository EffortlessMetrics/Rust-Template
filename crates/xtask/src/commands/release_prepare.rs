use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

use super::versioning::{VersionInfo, VersionManifest, apply_changes, plan_changes};

pub fn run(version: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{}", format!("📦 Preparing release {} (dry run)...", version).blue().bold());
    } else {
        println!("{}", format!("📦 Preparing release {}...", version).blue().bold());
    }
    println!();

    // Validate version format and create VersionInfo
    let version_info = VersionInfo::new(version).context("Invalid version format")?;

    // Load the version manifest
    let manifest = VersionManifest::load().context("Failed to load version manifest")?;

    // Plan changes from manifest
    let edits = plan_changes(&version_info, &manifest)?;

    // Apply version updates
    if !edits.is_empty() {
        apply_changes(&edits, dry_run)?;
    } else {
        println!("  No version changes needed (already up to date)");
    }

    // Handle CHANGELOG specially (insert skeleton, not replace)
    if !dry_run {
        if insert_changelog_skeleton(version, &version_info.date)? {
            println!("  {} CHANGELOG.md (skeleton inserted)", "✓".green());
        }
    } else {
        println!("\n{}", "CHANGELOG.md:".cyan());
        println!("  Would insert new release section for [{}]", version);
    }

    // Summary
    println!();
    if dry_run {
        println!("{}", "This was a dry run. No files were modified.".yellow().bold());
        println!("Run without --dry-run to apply changes.");
    } else {
        let version_full = format!("v{}", version);
        println!("{}", "Next steps:".bold());
        println!("  1. Fill in {} entry", "CHANGELOG.md".cyan());
        println!("  2. Review all changes: {}", "git diff".cyan());
        println!("  3. Run: {}", "cargo xtask docs-check".cyan());
        println!("  4. Run: {}", "cargo xtask selftest".cyan());
        println!("  5. Commit: {}", format!("git commit -am 'Release {}'", version_full).cyan());
        println!(
            "  6. Tag: {}",
            format!("git tag -a {} -m 'Release {}'", version_full, version).cyan()
        );
    }

    Ok(())
}

fn insert_changelog_skeleton(version: &str, date: &str) -> Result<bool> {
    let path = "CHANGELOG.md";
    let content = fs::read_to_string(path)?;

    // Find [Unreleased] and insert new version after it
    let skeleton = format!(
        r#"
## [{}] - {}

### Added

- 

### Changed

- 

### Fixed

- 
"#,
        version, date
    );

    let new_content =
        content.replace("## [Unreleased]", &format!("## [Unreleased]\n\n(empty)\n{}", skeleton));

    fs::write(path, new_content)?;
    Ok(true)
}
