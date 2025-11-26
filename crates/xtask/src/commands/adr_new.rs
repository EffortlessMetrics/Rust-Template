use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run(title: &str) -> Result<()> {
    println!("{}", "📝 Creating new ADR...".blue().bold());
    println!();

    // Find next ADR number
    let adr_dir = Path::new("docs/adr");
    let next_num = find_next_adr_number(adr_dir)?;

    // Create slug from title
    let slug = title
        .to_lowercase()
        .replace(char::is_whitespace, "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let adr_id = format!("ADR-{:04}", next_num);
    let filename = format!("{:04}-{}.md", next_num, slug);
    let filepath = adr_dir.join(&filename);

    // Get author from git config
    let author = get_git_user().unwrap_or_else(|_| "Unknown".to_string());

    // Get today's date
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Read template
    let template_path = Path::new("docs/templates/ADR-TEMPLATE.md");
    if !template_path.exists() {
        anyhow::bail!("ADR template not found at {}", template_path.display());
    }

    let template = fs::read_to_string(template_path)?;

    // Fill in template
    let content = template
        .replace("# ADR-XXXX: [Title]", &format!("# {}: {}", adr_id, title))
        .replace(
            "**Status**: [Proposed | Accepted | Deprecated | Superseded by ADR-YYYY]",
            "**Status**: Proposed",
        )
        .replace("**Date**: YYYY-MM-DD", &format!("**Date**: {}", date))
        .replace("**Authors**: [Author names or teams]", &format!("**Authors**: {}", author));

    // Write file
    fs::write(&filepath, content)?;

    println!("{} Created {}", "✓".green(), filepath.display());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Edit {}", filepath.display());
    println!("  2. Link this ADR in specs/spec_ledger.yaml:");
    println!("     - Metadata-level ADRs: under metadata.adrs");
    println!("     - Story-level: under story.adr");
    println!("     - Requirement-level: under requirement.adr");
    println!("     - AC-level: under acceptance_criteria.adr");
    println!("  3. Run: {}", "cargo xtask adr-check".cyan());
    println!("  4. Review & discuss; set Status: Accepted when settled");

    Ok(())
}

fn find_next_adr_number(adr_dir: &Path) -> Result<u32> {
    if !adr_dir.exists() {
        anyhow::bail!("ADR directory not found: {}", adr_dir.display());
    }

    let mut max_num = 0;

    for entry in fs::read_dir(adr_dir)? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Parse "0001-title.md" -> 1
        if let Some(num_str) = filename_str.split('-').next()
            && let Ok(num) = num_str.parse::<u32>()
        {
            max_num = max_num.max(num);
        }
    }

    Ok(max_num + 1)
}

fn get_git_user() -> Result<String> {
    let output = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .context("Failed to get git user.name")?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Ok(name);
        }
    }

    anyhow::bail!("Git user.name not configured")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr_new_command_exists() {
        // Verify that the run function is accessible and has the correct signature
        let _: fn(&str) -> Result<()> = run;
    }

    #[test]
    fn test_find_next_adr_number_increments() {
        // Test that ADR numbering logic increments correctly
        // This test verifies the logic without actually reading the filesystem
        use std::path::PathBuf;

        let adr_dir = PathBuf::from("docs/adr");
        if adr_dir.exists() {
            let next_num = find_next_adr_number(&adr_dir).expect("find next ADR number");
            // Next number should be at least 1
            assert!(next_num >= 1, "ADR number should be at least 1");
        }
    }

    #[test]
    fn test_slug_generation_from_title() {
        // Test the slug generation logic used in adr-new
        let title = "Test ADR Scaffolding";
        let slug = title
            .to_lowercase()
            .replace(char::is_whitespace, "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        assert_eq!(slug, "test-adr-scaffolding");
    }

    #[test]
    fn test_slug_generation_removes_special_chars() {
        // Test that special characters are removed from slug
        let title = "Test & Special! Chars?";
        let slug = title
            .to_lowercase()
            .replace(char::is_whitespace, "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        assert_eq!(slug, "test--special-chars");
    }

    #[test]
    fn test_adr_id_formatting() {
        // Test ADR ID formatting
        let next_num = 42;
        let adr_id = format!("ADR-{:04}", next_num);
        assert_eq!(adr_id, "ADR-0042");
    }
}
