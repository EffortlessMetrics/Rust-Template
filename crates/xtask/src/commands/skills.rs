use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const SKILLS_DIR: &str = ".claude/skills";

#[derive(Debug, Serialize, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    allowed_tools: Option<Vec<String>>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml::Value>,
}

pub fn run_fmt() -> Result<()> {
    let root = std::env::current_dir()?;
    let skills_path = root.join(SKILLS_DIR);

    if !skills_path.exists() {
        return Ok(());
    }

    let mut changed = false;

    for entry in WalkDir::new(&skills_path).min_depth(2).max_depth(2) {
        let entry = entry?;
        if entry.file_name() != "SKILL.md" {
            continue;
        }

        let path = entry.path();
        let slug = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();

        if format_skill(&slug, path)? {
            println!("formatted {}", path.display());
            changed = true;
        }
    }

    if changed {
        // Return error to signal pre-commit that files were modified
        std::process::exit(1);
    }

    Ok(())
}

fn format_skill(slug: &str, path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    let (fm_str, body) = match split_frontmatter(&content) {
        Some((f, b)) => (f, b),
        None => {
            // Synthesize minimal frontmatter
            let fm = SkillFrontmatter {
                name: slug.to_string(),
                description: format!("Skill for {}. Please update description.", slug),
                allowed_tools: None,
                extra: BTreeMap::new(),
            };
            let fm_str = serde_yaml::to_string(&fm)?;
            let new_content = format!("---\n{}---\n\n{}", fm_str, content.trim_start());
            fs::write(path, new_content)?;
            return Ok(true);
        }
    };

    // Parse existing frontmatter to preserve values
    // We use a generic Value first to handle potential malformed structure gracefully during fmt?
    // Actually, let's try to parse into our struct to enforce structure,
    // but if it fails, we might want to skip or handle it.
    // For fmt, we want to be aggressive about fixing structure.

    // Let's use a BTreeMap to preserve everything, then reconstruct
    let mut fm_map: BTreeMap<String, serde_yaml::Value> = match serde_yaml::from_str(fm_str) {
        Ok(map) => map,
        Err(_) => return Ok(false), // Let lint catch invalid YAML
    };

    // Ensure required fields
    if !fm_map.contains_key("name") {
        fm_map.insert("name".to_string(), serde_yaml::Value::String(slug.to_string()));
    }
    if !fm_map.contains_key("description") {
        fm_map.insert(
            "description".to_string(),
            serde_yaml::Value::String(format!("Skill for {}. Please update description.", slug)),
        );
    }

    // Reconstruct with specific order
    let mut ordered_map = serde_yaml::Mapping::new();

    // Helper to move key from source to target
    let mut move_key = |key: &str| {
        if let Some(val) = fm_map.remove(key) {
            ordered_map.insert(serde_yaml::Value::String(key.to_string()), val);
        }
    };

    move_key("name");
    move_key("description");
    move_key("allowed-tools");

    // Add remaining keys
    for (k, v) in fm_map {
        ordered_map.insert(serde_yaml::Value::String(k), v);
    }

    let new_fm_str = serde_yaml::to_string(&ordered_map)?;

    // Normalize body: ensure exactly one blank line after frontmatter
    let body = body.trim_start();
    let new_content = format!("---\n{}---\n\n{}", new_fm_str, body);

    if new_content != content {
        fs::write(path, new_content)?;
        return Ok(true);
    }

    Ok(false)
}

pub fn run_lint() -> Result<()> {
    let root = std::env::current_dir()?;
    let skills_path = root.join(SKILLS_DIR);

    if !skills_path.exists() {
        return Ok(());
    }

    let mut any_errors = false;
    let name_re = Regex::new(r"^[a-z0-9-]{1,64}$")?;

    for entry in WalkDir::new(&skills_path).min_depth(2).max_depth(2) {
        let entry = entry?;
        if entry.file_name() != "SKILL.md" {
            continue;
        }

        let path = entry.path();
        let slug = path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();

        let errors = lint_skill(&slug, path, &name_re)?;
        if !errors.is_empty() {
            any_errors = true;
            let rel_path = path.strip_prefix(&root).unwrap_or(path);
            println!("[SKILL LINT] {}:", rel_path.display());
            for err in errors {
                println!("  - {}", err.red());
            }
            println!();
        }
    }

    if any_errors {
        std::process::exit(1);
    }

    Ok(())
}

fn lint_skill(slug: &str, path: &Path, name_re: &Regex) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    let content = fs::read_to_string(path)?;

    // Check for tabs in first few lines
    if content.lines().take(20).any(|l| l.contains('\t')) {
        errors.push("Tabs found in SKILL.md (YAML must use spaces).".to_string());
    }

    let (fm_str, body) = match split_frontmatter(&content) {
        Some((f, b)) => (f, b),
        None => {
            errors.push("Missing frontmatter '---' at line 1".to_string());
            return Ok(errors);
        }
    };

    let fm: serde_yaml::Value = match serde_yaml::from_str(fm_str) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("YAML parse error: {}", e));
            return Ok(errors);
        }
    };

    // Validate name
    match fm.get("name") {
        Some(serde_yaml::Value::String(name)) => {
            if !name_re.is_match(name) {
                errors.push(format!(
                    "frontmatter 'name' must match {} (got '{}').",
                    name_re.as_str(),
                    name
                ));
            }
            if name != slug {
                errors.push(format!(
                    "frontmatter 'name' ('{}') must equal directory slug ('{}').",
                    name, slug
                ));
            }
        }
        _ => errors.push("frontmatter 'name' must be a string.".to_string()),
    }

    // Validate description
    match fm.get("description") {
        Some(serde_yaml::Value::String(desc)) => {
            if desc.trim().is_empty() {
                errors.push("frontmatter 'description' must be a non-empty string.".to_string());
            } else if desc.len() > 1024 {
                errors.push("frontmatter 'description' must be ≤1024 characters.".to_string());
            }
        }
        _ => errors.push("frontmatter 'description' must be a non-empty string.".to_string()),
    }

    // Validate allowed-tools
    if let Some(tools) = fm.get("allowed-tools")
        && !tools.is_sequence()
    {
        errors.push("frontmatter 'allowed-tools' must be a YAML list if present.".to_string());
    }

    // Validate body
    if !body.contains('#') {
        errors.push("Markdown body should contain at least one heading (# …).".to_string());
    }

    Ok(errors)
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }

    // Find the closing ---
    // This is a bit tricky with iterators, let's do it simpler
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 3 && parts[0].trim().is_empty() {
        // parts[0] is empty (before first ---)
        // parts[1] is frontmatter
        // parts[2] is body
        return Some((parts[1], parts[2]));
    }

    None
}
