use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub(super) fn validate_skills_agents_alignment() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    let ledger_path = root.join("specs/spec_ledger.yaml");
    if !ledger_path.exists() {
        // Can't validate without spec_ledger
        return Ok(());
    }

    let ledger_content = fs::read_to_string(&ledger_path)
        .with_context(|| format!("Failed to read {}", ledger_path.display()))?;

    // Extract declared skills from spec_ledger
    // Look for AC-TPL-SKILLS-ALIGN-001 which lists expected skills
    let declared_skills = extract_declared_skills(&ledger_content);

    // Extract declared agents from spec_ledger
    // Agents are referenced in REQ-TPL-AGENTS-GOVERNANCE section
    let declared_agents = extract_declared_agents(&ledger_content);

    // Scan filesystem for actual skills
    let skills_dir = root.join(".claude/skills");
    let mut actual_skills: Vec<String> = Vec::new();
    if skills_dir.exists() {
        for entry in WalkDir::new(&skills_dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let skill_file = entry.path().join("SKILL.md");
                if skill_file.exists()
                    && let Some(name) = entry.file_name().to_str()
                {
                    actual_skills.push(name.to_string());
                }
            }
        }
    }

    // Scan filesystem for actual agents
    let agents_dir = root.join(".claude/agents");
    let mut actual_agents: Vec<String> = Vec::new();
    if agents_dir.exists() {
        for entry in WalkDir::new(&agents_dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false)
                    && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                {
                    actual_agents.push(name.to_string());
                }
            }
        }
    }

    let mut issues = Vec::new();

    // Check for orphaned skills (exist in directory but not declared)
    if !declared_skills.is_empty() {
        for skill in &actual_skills {
            if !declared_skills.contains(skill) {
                issues.push(format!(
                    "Skill '{}' exists in .claude/skills/ but is not declared in spec_ledger.yaml (AC-TPL-SKILLS-ALIGN-001)",
                    skill
                ));
            }
        }

        // Check for missing skills (declared but directory missing)
        for skill in &declared_skills {
            if !actual_skills.contains(skill) {
                issues.push(format!(
                    "Skill '{}' declared in spec_ledger.yaml but missing from .claude/skills/",
                    skill
                ));
            }
        }
    }

    // Check for orphaned agents (exist in directory but not declared)
    // Note: Agents governance is softer - we mainly check that agents exist
    // and are governed by REQ-TPL-AGENTS-GOVERNANCE requirements
    if !declared_agents.is_empty() {
        for agent in &actual_agents {
            if !declared_agents.contains(agent) {
                issues.push(format!(
                    "Agent '{}' exists in .claude/agents/ but may need spec_ledger coverage (REQ-TPL-AGENTS-GOVERNANCE)",
                    agent
                ));
            }
        }

        // Check for missing agents (declared but file missing)
        for agent in &declared_agents {
            if !actual_agents.contains(agent) {
                issues.push(format!(
                    "Agent '{}' referenced in spec_ledger.yaml but missing from .claude/agents/",
                    agent
                ));
            }
        }
    }

    // Also validate that basic governance docs exist
    let skills_governance = root.join("docs/SKILLS_GOVERNANCE.md");
    let agents_governance = root.join("docs/AGENTS_GOVERNANCE.md");

    if !actual_skills.is_empty() && !skills_governance.exists() {
        issues.push(
            "Skills exist but docs/SKILLS_GOVERNANCE.md is missing (AC-TPL-SKILLS-GOVERNANCE-001)"
                .to_string(),
        );
    }

    if !actual_agents.is_empty() && !agents_governance.exists() {
        issues.push(
            "Agents exist but docs/AGENTS_GOVERNANCE.md is missing (AC-TPL-AGENTS-GOVERNANCE-001)"
                .to_string(),
        );
    }

    if !issues.is_empty() {
        eprintln!();
        eprintln!("{}", "Skills/Agents alignment issues:".yellow().bold());
        for issue in &issues {
            eprintln!("  ⚠ {}", issue);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Ensure each Skill in .claude/skills/* is listed in AC-TPL-SKILLS-ALIGN-001");
        eprintln!("  • Ensure each Agent in .claude/agents/* has REQ/AC coverage");
        eprintln!("  • See: {}", "docs/SKILLS_GOVERNANCE.md".cyan());
        eprintln!("  • See: {}", "docs/AGENTS_GOVERNANCE.md".cyan());
        anyhow::bail!("{} alignment issue(s)", issues.len());
    }

    Ok(())
}

/// Extract declared skill names from spec_ledger.yaml content.
/// Looks for AC-TPL-SKILLS-ALIGN-001 which lists expected skills.
pub(super) fn extract_declared_skills(content: &str) -> Vec<String> {
    let mut skills = Vec::new();

    // Look for the AC-TPL-SKILLS-ALIGN-001 section which contains:
    // (bootstrap-dev-env, governed-feature-dev, governed-maintenance,
    // governed-release, governed-governance-debug)
    let skill_pattern = Regex::new(
        r"(?:bootstrap-dev-env|governed-feature-dev|governed-maintenance|governed-release|governed-governance-debug)"
    ).expect("valid regex");

    // Find the AC-TPL-SKILLS-ALIGN-001 section
    let mut in_align_section = false;
    for line in content.lines() {
        if line.contains("AC-TPL-SKILLS-ALIGN-001") {
            in_align_section = true;
            continue;
        }
        // Look for the next AC or requirement to mark end of section
        if in_align_section
            && (line.trim().starts_with("- id: AC-")
                || line.trim().starts_with("- id: REQ-")
                || line.trim().starts_with("- id: US-"))
        {
            break;
        }

        if in_align_section {
            // Extract skill names from this section
            for cap in skill_pattern.find_iter(line) {
                let skill_name = cap.as_str().to_string();
                if !skills.contains(&skill_name) {
                    skills.push(skill_name);
                }
            }
        }
    }

    // Also look for any explicit skills: list in the ledger
    // Pattern: skills: [skill1, skill2] or skills:\n  - skill1\n  - skill2
    let skills_list_re = Regex::new(r#"\.claude/skills/([a-z0-9-]+)"#).expect("valid regex");
    for cap in skills_list_re.captures_iter(content) {
        if let Some(skill_name) = cap.get(1) {
            let name = skill_name.as_str().to_string();
            if !skills.contains(&name) {
                skills.push(name);
            }
        }
    }

    skills
}

/// Extract declared agent names from spec_ledger.yaml content.
/// Looks for REQ-TPL-AGENTS-GOVERNANCE and related sections.
pub(super) fn extract_declared_agents(content: &str) -> Vec<String> {
    let mut agents = Vec::new();

    // Look for agent references in the form .claude/agents/agent-name.md
    let agents_re = Regex::new(r#"\.claude/agents/([a-z0-9-]+)\.md"#).expect("valid regex");
    for cap in agents_re.captures_iter(content) {
        if let Some(agent_name) = cap.get(1) {
            let name = agent_name.as_str().to_string();
            if !agents.contains(&name) {
                agents.push(name);
            }
        }
    }

    // Also look for agent mentions in the agents/* pattern
    let agents_pattern_re = Regex::new(r#"agents/\*"#).expect("valid regex");
    if agents_pattern_re.is_match(content) {
        // The pattern .claude/agents/* is mentioned, which means agents are governed
        // but specific names are not enumerated in spec_ledger
        // In this case, we return empty to avoid false positives
        // The governance is established by REQ-TPL-AGENTS-GOVERNANCE
    }

    agents
}
