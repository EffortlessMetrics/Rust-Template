use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ServiceInitArgs {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Option<Vec<String>>,
    pub register_fork: bool,
}

/// Service metadata structure for service_metadata.yaml.
/// Future: Used when implementing service metadata persistence and validation.
/// See TASK-DX-SERVICE-METADATA for planned metadata features.
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct ServiceMetadata {
    service_id: String,
    display_name: String,
    description: String,
    template_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ownership: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lifecycle: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

pub fn run(args: ServiceInitArgs) -> Result<()> {
    println!("{}", "🚀 Initializing service branding...".blue().bold());
    println!();

    // Validate ID format (kebab-case)
    validate_service_id(&args.id)?;

    // Track changes for summary
    let mut changes = Vec::new();

    // 1. Update service_metadata.yaml
    update_service_metadata(&args, &mut changes)?;

    // 2. Update README.md
    update_readme(&args, &mut changes)?;

    // 3. Optionally append to fork_registry.yaml
    if args.register_fork {
        update_fork_registry(&args, &mut changes)?;
    }

    // Print summary
    print_summary(&args, &changes);

    Ok(())
}

fn validate_service_id(id: &str) -> Result<()> {
    // kebab-case pattern: lowercase letters, numbers, and hyphens
    // Must start with a letter, cannot end with hyphen
    let kebab_case_pattern = Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap();

    if !kebab_case_pattern.is_match(id) {
        anyhow::bail!(
            "Invalid service ID '{}'. Must be kebab-case (lowercase letters, numbers, hyphens). Examples: 'my-service', 'hr-hub', 'data-api'",
            id
        );
    }

    Ok(())
}

fn update_service_metadata(args: &ServiceInitArgs, changes: &mut Vec<String>) -> Result<()> {
    let metadata_path = Path::new("specs/service_metadata.yaml");

    if !metadata_path.exists() {
        anyhow::bail!("service_metadata.yaml not found at {}", metadata_path.display());
    }

    // Read current metadata
    let content =
        fs::read_to_string(metadata_path).context("Failed to read service_metadata.yaml")?;

    let mut metadata: serde_yaml::Value =
        serde_yaml::from_str(&content).context("Failed to parse service_metadata.yaml")?;

    // Check if changes are needed (idempotency)
    let current_id = metadata.get("service_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let current_name = metadata.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let current_desc = metadata.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut modified = false;

    if current_id.as_deref() != Some(&args.id) {
        metadata["service_id"] = serde_yaml::Value::String(args.id.clone());
        modified = true;
    }

    if current_name.as_deref() != Some(&args.name) {
        metadata["display_name"] = serde_yaml::Value::String(args.name.clone());
        modified = true;
    }

    // Handle description (could be a string or a multi-line string)
    let desc_matches =
        current_desc.as_ref().map(|d| d.trim() == args.description.trim()).unwrap_or(false);
    if !desc_matches {
        metadata["description"] = serde_yaml::Value::String(args.description.clone());
        modified = true;
    }

    // Update tags if provided
    if let Some(ref new_tags) = args.tags {
        metadata["tags"] = serde_yaml::to_value(new_tags).context("Failed to serialize tags")?;
        modified = true;
    }

    if modified {
        // Write back to file
        let updated_content =
            serde_yaml::to_string(&metadata).context("Failed to serialize updated metadata")?;

        fs::write(metadata_path, updated_content)
            .context("Failed to write service_metadata.yaml")?;

        changes.push(format!("Updated {}", metadata_path.display()));
    } else {
        changes.push(format!("No changes needed in {}", metadata_path.display()));
    }

    Ok(())
}

fn update_readme(args: &ServiceInitArgs, changes: &mut Vec<String>) -> Result<()> {
    let readme_path = Path::new("README.md");

    if !readme_path.exists() {
        anyhow::bail!("README.md not found at {}", readme_path.display());
    }

    let content = fs::read_to_string(readme_path).context("Failed to read README.md")?;

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        anyhow::bail!("README.md is empty");
    }

    // Find the first markdown heading (title)
    let mut title_line_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            title_line_idx = Some(idx);
            break;
        }
    }

    let title_line_idx = title_line_idx
        .ok_or_else(|| anyhow::anyhow!("Could not find title (# heading) in README.md"))?;

    // Find the description (first non-empty paragraph after frontmatter/title)
    // Skip version/kernel info lines that start with **
    let mut desc_line_idx = None;
    for (idx, line) in lines.iter().enumerate().skip(title_line_idx + 1) {
        let trimmed = line.trim();
        // Skip empty lines and lines starting with ** (metadata)
        if !trimmed.is_empty() && !trimmed.starts_with("**") && !trimmed.starts_with(">") {
            desc_line_idx = Some(idx);
            break;
        }
    }

    let desc_line_idx = desc_line_idx
        .ok_or_else(|| anyhow::anyhow!("Could not find description line in README.md"))?;

    // Check if changes are needed (idempotency)
    let current_title = lines[title_line_idx].trim_start_matches("# ").trim();
    let current_desc = lines[desc_line_idx].trim();

    let mut new_lines = lines.clone();
    let mut modified = false;

    // Extract version info from title if it exists (e.g., "Title (v3.3.3)")
    let version_suffix = if current_title.contains('(') && current_title.contains(')') {
        // Find the last opening parenthesis
        if let Some(pos) = current_title.rfind('(') {
            format!(" {}", &current_title[pos..])
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let new_title = format!("# {}{}", args.name, version_suffix);

    if lines[title_line_idx] != new_title {
        new_lines[title_line_idx] = &new_title;
        modified = true;
    }

    if current_desc != args.description {
        new_lines[desc_line_idx] = &args.description;
        modified = true;
    }

    if modified {
        // We need to reconstruct the content with owned strings
        let mut final_lines: Vec<String> = Vec::new();
        for (idx, line) in new_lines.iter().enumerate() {
            if idx == title_line_idx {
                final_lines.push(new_title.clone());
            } else if idx == desc_line_idx {
                final_lines.push(args.description.clone());
            } else {
                final_lines.push(line.to_string());
            }
        }

        let updated_content = final_lines.join("\n");
        // Preserve trailing newline if original had one
        let final_content = if content.ends_with('\n') {
            format!("{}\n", updated_content)
        } else {
            updated_content
        };

        fs::write(readme_path, final_content).context("Failed to write README.md")?;

        changes.push(format!("Updated {}", readme_path.display()));
    } else {
        changes.push(format!("No changes needed in {}", readme_path.display()));
    }

    Ok(())
}

fn update_fork_registry(_args: &ServiceInitArgs, changes: &mut Vec<String>) -> Result<()> {
    // For now, just add a placeholder
    // Full implementation would append a fork entry to forks/fork_registry.yaml
    changes.push(
        "Fork registry update: Not implemented yet (use 'cargo xtask fork-register')".to_string(),
    );

    println!(
        "{}",
        "ℹ️  Note: Use 'cargo xtask fork-register' to register this fork in the registry".yellow()
    );

    Ok(())
}

fn print_summary(args: &ServiceInitArgs, changes: &Vec<String>) {
    println!();
    println!("{}", "✓ Service initialization complete!".green().bold());
    println!();
    println!("{}", "Changes applied:".bold());
    for change in changes {
        println!("  • {}", change);
    }
    println!();
    println!("{}", "Service identity:".bold());
    println!("  Service ID:  {}", args.id.cyan());
    println!("  Name:        {}", args.name.cyan());
    println!("  Description: {}", args.description.cyan());
    if let Some(ref tags) = args.tags {
        println!("  Tags:        {}", tags.join(", ").cyan());
    }
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Review changes: git diff");
    println!("  2. Update ownership and links in specs/service_metadata.yaml");
    println!("  3. Customize the rest of README.md for your service");
    println!("  4. Run: {}", "cargo xtask selftest".cyan());
    println!("  5. Commit: git add -A && git commit -m \"chore: initialize service branding\"");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_service_id_valid() {
        assert!(validate_service_id("my-service").is_ok());
        assert!(validate_service_id("hr-hub").is_ok());
        assert!(validate_service_id("data-api").is_ok());
        assert!(validate_service_id("service123").is_ok());
        assert!(validate_service_id("my-service-v2").is_ok());
    }

    #[test]
    fn test_validate_service_id_invalid() {
        // Must start with letter
        assert!(validate_service_id("123-service").is_err());
        assert!(validate_service_id("-service").is_err());

        // No uppercase
        assert!(validate_service_id("MyService").is_err());
        assert!(validate_service_id("my-Service").is_err());

        // No underscores
        assert!(validate_service_id("my_service").is_err());

        // Cannot end with hyphen
        assert!(validate_service_id("my-service-").is_err());

        // No spaces
        assert!(validate_service_id("my service").is_err());

        // No special characters
        assert!(validate_service_id("my-service!").is_err());
        assert!(validate_service_id("my@service").is_err());
    }

    #[test]
    fn test_validate_service_id_edge_cases() {
        // Single letter is valid
        assert!(validate_service_id("a").is_ok());

        // Single word is valid
        assert!(validate_service_id("service").is_ok());

        // Multiple hyphens
        assert!(validate_service_id("my-really-long-service-name").is_ok());

        // Numbers in the middle
        assert!(validate_service_id("service-v2-beta3").is_ok());
    }
}
