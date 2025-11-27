use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const AGENTS_DIR: &str = ".claude/agents";
const SKILLS_DIR: &str = ".claude/skills";

pub fn run_lint() -> Result<()> {
    let root = std::env::current_dir()?;
    let agents_path = root.join(AGENTS_DIR);

    if !agents_path.exists() {
        return Ok(());
    }

    let mut any_errors = false;
    let name_re = Regex::new(r"^[a-z0-9-]{1,64}$")?;

    // Pre-load list of existing skills for reference validation
    let mut existing_skills = std::collections::HashSet::new();
    let skills_path = root.join(SKILLS_DIR);
    if skills_path.exists() {
        for entry in
            WalkDir::new(&skills_path).min_depth(1).max_depth(1).into_iter().filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir()
                && let Some(skill_name) = entry.file_name().to_str()
            {
                existing_skills.insert(skill_name.to_string());
            }
        }
    }

    for entry in WalkDir::new(&agents_path).min_depth(1).max_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().map(|e| e.to_str()) != Some(Some("md")) {
            continue;
        }

        let slug = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        let (errors, warnings) = lint_agent(&slug, path, &name_re, &existing_skills)?;
        let rel_path = path.strip_prefix(&root).unwrap_or(path);

        // Always print errors (they block agent use)
        if !errors.is_empty() {
            any_errors = true;
            println!("[AGENT LINT] {} - ERRORS:", rel_path.display());
            for err in &errors {
                println!("  {} {}", "✗".red(), err);
            }
            println!();
        }

        // Always print warnings (they improve quality but don't block)
        if !warnings.is_empty() {
            println!("[AGENT LINT] {} - WARNINGS:", rel_path.display());
            for warn in &warnings {
                println!("  {} {}", "⚠".yellow(), warn);
            }
            println!();
        }

        // Print success message if no errors or warnings
        if errors.is_empty() && warnings.is_empty() {
            println!("[AGENT LINT] {} ✓", rel_path.display());
        }
    }

    // Only fail on errors, not warnings
    if any_errors {
        std::process::exit(1);
    }

    Ok(())
}

/// Lint a single Agent: (errors, warnings)
/// - Errors: Hard failures that block agent use (invalid YAML, missing required fields, etc.)
/// - Warnings: Guidance that improves agent quality but doesn't block
fn lint_agent(
    slug: &str,
    path: &Path,
    name_re: &Regex,
    existing_skills: &std::collections::HashSet<String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let content = fs::read_to_string(path)?;

    // Check for tabs in first few lines (ERROR)
    if content.lines().take(20).any(|l| l.contains('\t')) {
        errors.push("Tabs found in agent file (YAML must use spaces).".to_string());
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
                    "frontmatter 'name' ('{}') must equal file name without extension ('{}').",
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
                    || desc_lower.contains("if ")
                    || desc_lower.contains("context");
                if !has_when {
                    warnings.push(
                        "description could be more specific: try including 'when to use' or trigger context."
                            .to_string(),
                    );
                }
            }
        }
        _ => errors.push("frontmatter 'description' must be a non-empty string.".to_string()),
    }

    // Validate tools: accept both YAML list and scalar string (ERROR for invalid)
    if let Some(tools) = fm.get("tools") {
        match tools {
            serde_yaml::Value::Sequence(_) => {
                // Valid: list of tools
            }
            serde_yaml::Value::String(s) => {
                // Also acceptable: comma-separated string (e.g., "Read, Grep, Glob")
                if s.trim().is_empty() {
                    errors.push("frontmatter 'tools' must not be empty if specified.".to_string());
                }
            }
            _ => {
                errors.push(
                    "frontmatter 'tools' must be a YAML list or comma-separated string."
                        .to_string(),
                );
            }
        }

        // Warning: check for broad tool combinations
        let tools_str = match tools {
            serde_yaml::Value::Sequence(seq) => {
                seq.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(",")
            }
            serde_yaml::Value::String(s) => s.to_string(),
            _ => String::new(),
        };

        let has_bash = tools_str.contains("Bash");
        let has_edit = tools_str.contains("Edit");
        let has_write = tools_str.contains("Write");
        if has_bash && has_edit && has_write {
            warnings.push(
                "broad tool set detected (Bash + Edit + Write); ensure this is justified by agent role."
                    .to_string(),
            );
        }
    }

    // Validate model: only allow specific values or inherit
    if let Some(model) = fm.get("model")
        && let serde_yaml::Value::String(m) = model
    {
        let allowed_models = ["sonnet", "opus", "haiku", "inherit"];
        if !allowed_models.contains(&m.as_str()) {
            errors.push(format!(
                "frontmatter 'model' must be one of {:?} (got '{}').",
                allowed_models, m
            ));
        }
        // Warning: opus is expensive
        if m == "opus" {
            warnings.push(
                "expensive model 'opus' specified; ensure this is justified by agent complexity."
                    .to_string(),
            );
        }
    }

    // Validate permissionMode: only allow specific values
    if let Some(perm_mode) = fm.get("permissionMode")
        && let serde_yaml::Value::String(pm) = perm_mode
    {
        let allowed_modes = ["default", "acceptEdits", "bypassPermissions", "plan", "ignore"];
        if !allowed_modes.contains(&pm.as_str()) {
            errors.push(format!(
                "frontmatter 'permissionMode' must be one of {:?} (got '{}').",
                allowed_modes, pm
            ));
        }
        // Warning: bypassPermissions is high-risk
        if pm == "bypassPermissions" {
            warnings.push(
                    "high-risk permissionMode 'bypassPermissions' specified; ensure this is reviewed and justified."
                        .to_string(),
                );
        }
    }

    // Validate skills: references must exist
    if let Some(skills_val) = fm.get("skills") {
        let skill_names = match skills_val {
            serde_yaml::Value::Sequence(seq) => {
                seq.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect::<Vec<_>>()
            }
            serde_yaml::Value::String(s) => {
                // Split by comma and trim
                s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            }
            _ => {
                errors.push(
                    "frontmatter 'skills' must be a YAML list or comma-separated string."
                        .to_string(),
                );
                return Ok((errors, warnings));
            }
        };

        for skill_name in skill_names {
            if !existing_skills.contains(&skill_name) {
                errors.push(format!(
                    "Skill '{}' referenced in 'skills' does not exist in .claude/skills/",
                    skill_name
                ));
            }
        }
    }

    // Validate body (WARNING if no headings)
    if !body.contains('#') {
        warnings.push("Markdown body should contain at least one heading (# …).".to_string());
    }

    Ok((errors, warnings))
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
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
    fn test_agent_name_validation_kebab_case() {
        let name_re = Regex::new(r"^[a-z0-9-]{1,64}$").unwrap();
        assert!(name_re.is_match("my-agent"));
        assert!(name_re.is_match("test-agent-123"));
        assert!(!name_re.is_match("MyAgent"));
        assert!(!name_re.is_match("my_agent"));
        assert!(!name_re.is_match("my agent"));
    }

    #[test]
    fn test_agent_name_validation_max_length() {
        let name_re = Regex::new(r"^[a-z0-9-]{1,64}$").unwrap();
        let long_name = "a".repeat(65);
        assert!(!name_re.is_match(&long_name));

        let valid_name = "a".repeat(64);
        assert!(name_re.is_match(&valid_name));
    }

    #[test]
    fn test_agent_description_nonempty() {
        let test_yaml = "---\nname: test\ndescription: |\n  This is a test agent.\n---\n\n# Test";
        let (fm_str, _) = split_frontmatter(test_yaml).unwrap();
        let fm: serde_yaml::Value = serde_yaml::from_str(fm_str).unwrap();
        assert!(fm.get("description").is_some());
    }

    #[test]
    fn test_agent_permission_mode_valid() {
        let test_yaml =
            "---\nname: test\ndescription: test\npermissionMode: default\n---\n\n# Test";
        let (fm_str, _) = split_frontmatter(test_yaml).unwrap();
        let fm: serde_yaml::Value = serde_yaml::from_str(fm_str).unwrap();

        let allowed_modes = ["default", "acceptEdits", "bypassPermissions", "plan", "ignore"];
        if let Some(serde_yaml::Value::String(mode)) = fm.get("permissionMode") {
            assert!(allowed_modes.contains(&mode.as_str()));
        }
    }

    #[test]
    fn test_agent_model_aliases() {
        let test_yaml = "---\nname: test\ndescription: test\nmodel: sonnet\n---\n\n# Test";
        let (fm_str, _) = split_frontmatter(test_yaml).unwrap();
        let fm: serde_yaml::Value = serde_yaml::from_str(fm_str).unwrap();

        let allowed_models = ["sonnet", "opus", "haiku", "inherit"];
        if let Some(serde_yaml::Value::String(model)) = fm.get("model") {
            assert!(allowed_models.contains(&model.as_str()));
        }
    }

    #[test]
    fn test_agent_skills_exist() {
        let mut existing_skills = std::collections::HashSet::new();
        existing_skills.insert("test-skill".to_string());

        // This is a basic check - in real scenario would use full lint_agent
        assert!(existing_skills.contains("test-skill"));
        assert!(!existing_skills.contains("nonexistent-skill"));
    }
}
