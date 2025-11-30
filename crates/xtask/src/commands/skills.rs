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

        let (errors, warnings) = lint_skill(&slug, path, &name_re)?;
        let rel_path = path.strip_prefix(&root).unwrap_or(path);

        // Always print errors (they block Skill use)
        if !errors.is_empty() {
            any_errors = true;
            println!("[SKILL LINT] {} - ERRORS:", rel_path.display());
            for err in &errors {
                println!("  {} {}", "✗".red(), err);
            }
            println!();
        }

        // Always print warnings (they improve quality but don't block)
        if !warnings.is_empty() {
            println!("[SKILL LINT] {} - WARNINGS:", rel_path.display());
            for warn in &warnings {
                println!("  {} {}", "⚠".yellow(), warn);
            }
            println!();
        }

        // Print success message if no errors or warnings
        if errors.is_empty() && warnings.is_empty() {
            println!("[SKILL LINT] {} ✓", rel_path.display());
        }
    }

    // Only fail on errors, not warnings
    if any_errors {
        std::process::exit(1);
    }

    Ok(())
}

/// Lint a single Skill: (errors, warnings)
/// - Errors: Hard failures that block Skill use (invalid YAML, missing required fields, etc.)
/// - Warnings: Guidance that improves Skill quality but doesn't block (could be more specific, etc.)
fn lint_skill(slug: &str, path: &Path, name_re: &Regex) -> Result<(Vec<String>, Vec<String>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let content = fs::read_to_string(path)?;

    // Check for tabs in first few lines (ERROR)
    if content.lines().take(20).any(|l| l.contains('\t')) {
        errors.push("Tabs found in SKILL.md (YAML must use spaces).".to_string());
    }

    let (fm_str, body) = match split_frontmatter(&content) {
        Some((f, b)) => (f, b),
        None => {
            errors.push("Missing frontmatter '---' at line 1".to_string());
            return Ok((errors, warnings));
        }
    };

    let fm: serde_yaml::Value = match serde_yaml::from_str(fm_str) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("YAML parse error: {}", e));
            return Ok((errors, warnings));
        }
    };

    // Validate name (ERROR)
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

    // Validate description (ERROR if missing/too long, WARNING if vague)
    match fm.get("description") {
        Some(serde_yaml::Value::String(desc)) => {
            if desc.trim().is_empty() {
                errors.push("frontmatter 'description' must be a non-empty string.".to_string());
            } else if desc.len() > 1024 {
                errors.push("frontmatter 'description' must be ≤1024 characters.".to_string());
            } else {
                // Soft check: description should include both "what" and "when"
                let desc_lower = desc.to_lowercase();
                let has_when = desc_lower.contains("when")
                    || desc_lower.contains("use when")
                    || desc_lower.contains("trigger")
                    || desc_lower.contains("if ");
                if !has_when {
                    warnings.push(
                        "description could be more specific: try including 'when to use' or trigger phrases."
                            .to_string(),
                    );
                }
            }
        }
        _ => errors.push("frontmatter 'description' must be a non-empty string.".to_string()),
    }

    // Validate allowed-tools: accept both YAML list and scalar string (ERROR for invalid)
    if let Some(tools) = fm.get("allowed-tools") {
        match tools {
            serde_yaml::Value::Sequence(_) => {
                // Valid: list of tools
            }
            serde_yaml::Value::String(s) => {
                // Also acceptable: comma-separated string (e.g., "Read, Grep, Glob")
                if s.trim().is_empty() {
                    errors.push("frontmatter 'allowed-tools' must not be empty.".to_string());
                }
            }
            _ => {
                errors.push(
                    "frontmatter 'allowed-tools' must be a YAML list or comma-separated string."
                        .to_string(),
                );
            }
        }
    }

    // Validate body (WARNING if no headings)
    if !body.contains('#') {
        warnings.push("Markdown body should contain at least one heading (# …).".to_string());
    }

    // Validate no hardcoded secrets (ERROR)
    validate_no_secrets(&content, &mut errors);

    Ok((errors, warnings))
}

/// Check for patterns that suggest hardcoded secrets
fn validate_no_secrets(content: &str, errors: &mut Vec<String>) {
    // Patterns that suggest hardcoded secrets (case-insensitive)
    // These must be preceded by word boundaries or special characters to avoid false positives
    let secret_patterns = [
        "credentials.json",
        "secrets.yaml",
        "secrets.yml",
        "api_key",
        "api-key",
        "password",
        "api key",
        "token",
        "secret",
        "passwd",
        ".env",
    ];

    for (line_num, line) in content.lines().enumerate() {
        let line_lower = line.to_lowercase();

        // Check if line looks like it contains a secret assignment
        let has_assignment = line_lower.contains("=") || line_lower.contains(":");
        if !has_assignment {
            continue;
        }

        for pattern in &secret_patterns {
            if is_secret_pattern_match(&line_lower, pattern) {
                errors.push(format!(
                    "Hardcoded secret detected at line {}: contains pattern '{}'",
                    line_num + 1,
                    pattern
                ));
                break;
            }
        }

        // Special handling for sk- pattern: only match if it looks like an actual secret key
        if is_secret_pattern_match(&line_lower, "sk-") {
            errors.push(format!(
                "Hardcoded secret detected at line {}: contains pattern 'sk-'",
                line_num + 1
            ));
        }
    }
}

/// Check if a pattern match is actually a secret pattern
/// Avoids false positives by checking word boundaries
fn is_secret_pattern_match(line: &str, pattern: &str) -> bool {
    if !line.contains(pattern) {
        return false;
    }

    // For short patterns, require word boundaries to avoid false positives like "task-" containing "sk-"
    if pattern.len() <= 4 {
        // Check each occurrence
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(pattern) {
            let actual_pos = search_pos + pos;
            // Check character before pattern
            let before_ok = actual_pos == 0
                || matches!(line.chars().nth(actual_pos - 1), Some(c) if !c.is_alphanumeric());
            // Check character after pattern
            let after_ok = actual_pos + pattern.len() >= line.len()
                || matches!(line.chars().nth(actual_pos + pattern.len()), Some(c) if !c.is_alphanumeric());

            if before_ok && after_ok {
                return true;
            }
            search_pos = actual_pos + 1;
        }
        false
    } else {
        true
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_no_secrets_detects_api_key() {
        let mut errors = Vec::new();
        validate_no_secrets("api_key = sk-abc123", &mut errors);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("api_key"));
    }

    #[test]
    fn test_validate_no_secrets_detects_password() {
        let mut errors = Vec::new();
        validate_no_secrets("password: \"secret-password\"", &mut errors);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("password"));
    }

    #[test]
    fn test_validate_no_secrets_detects_token() {
        let mut errors = Vec::new();
        validate_no_secrets("token = abc123def456", &mut errors);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("token"));
    }

    #[test]
    fn test_validate_no_secrets_allows_mention_without_assignment() {
        let mut errors = Vec::new();
        // Mentioning "token" in documentation without assignment should be okay
        validate_no_secrets("This skill uses the token from environment variables", &mut errors);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_no_secrets_detects_credentials_file() {
        let mut errors = Vec::new();
        validate_no_secrets("credentials.json: /path/to/credentials", &mut errors);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("credentials.json"));
    }

    #[test]
    fn test_skill_name_validation_kebab_case() {
        let name_re = Regex::new(r"^[a-z0-9-]{1,64}$").unwrap();
        assert!(name_re.is_match("my-skill"));
        assert!(name_re.is_match("test-skill-123"));
        assert!(!name_re.is_match("MySkill"));
        assert!(!name_re.is_match("my_skill"));
        assert!(!name_re.is_match("my skill"));
    }

    #[test]
    fn test_skill_name_validation_max_length() {
        let name_re = Regex::new(r"^[a-z0-9-]{1,64}$").unwrap();
        let long_name = "a".repeat(65);
        assert!(!name_re.is_match(&long_name));

        let valid_name = "a".repeat(64);
        assert!(name_re.is_match(&valid_name));
    }
}
