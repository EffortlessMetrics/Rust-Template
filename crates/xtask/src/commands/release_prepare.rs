use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run(version: &str) -> Result<()> {
    println!("{}", format!("📦 Preparing release {}...", version).blue().bold());
    println!();

    // Validate version format
    if !version.chars().next().unwrap_or('0').is_numeric() {
        anyhow::bail!("Version should be format X.Y.Z (e.g., 2.5.0)");
    }

    let version_full = format!("v{}", version);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut updated = Vec::new();

    // Update spec_ledger.yaml
    if update_file_version("specs/spec_ledger.yaml", "template_version:", &version)? {
        updated.push("specs/spec_ledger.yaml");
    }

    // Update README.md
    if update_readme_version(&version)? {
        updated.push("README.md");
    }

    // Update CLAUDE.md
    if update_claude_version(&version, &today)? {
        updated.push("CLAUDE.md");
    }

    // Insert CHANGELOG skeleton
    if insert_changelog_skeleton(&version, &today)? {
        updated.push("CHANGELOG.md");
    }

    println!();
    println!("{} Updated {} file(s):", "✓".green(), updated.len());
    for file in &updated {
        println!("  • {}", file);
    }

    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Fill in {} entry", "CHANGELOG.md".cyan());
    println!("  2. Review all changes: {}", "git diff".cyan());
    println!("  3. Run: {}", "cargo xtask release-verify".cyan());
    println!("  4. Commit: {}", format!("git commit -am 'Release {}'", version_full).cyan());
    println!(
        "  5. Tag: {}",
        format!("git tag -a {} -m 'Release {}'", version_full, version).cyan()
    );

    Ok(())
}

fn update_file_version(path: &str, marker: &str, version: &str) -> Result<bool> {
    if !Path::new(path).exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)?;
    let mut modified = false;

    let new_content: String = content
        .lines()
        .map(|line| {
            if line.trim().starts_with(marker) {
                modified = true;
                format!("  {}: \"{}\"", marker.trim_end_matches(':'), version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if modified {
        fs::write(path, new_content + "\n")?;
    }

    Ok(modified)
}

fn update_readme_version(version: &str) -> Result<bool> {
    let path = "README.md";
    let content = fs::read_to_string(path)?;
    let version_line = format!("**Current Template Version:** v{}", version);

    let new_content = content
        .lines()
        .map(|line| {
            if line.contains("Current Template Version") {
                version_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(path, new_content + "\n")?;
    Ok(true)
}

fn update_claude_version(version: &str, date: &str) -> Result<bool> {
    let path = "CLAUDE.md";
    let content = fs::read_to_string(path)?;

    let new_content = content
        .lines()
        .map(|line| {
            if line.contains("Template Version:") {
                format!("**Template Version:** v{}", version)
            } else if line.contains("Last Updated:") {
                format!("**Last Updated:** {}", date)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(path, new_content + "\n")?;
    Ok(true)
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
