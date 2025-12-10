//! Suggest BDD scenarios for a given AC.
//!
//! This command looks up an AC in spec_ledger.yaml and generates
//! a skeleton Gherkin scenario based on the AC text.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::commands::ac_parsing::{AcDetails, get_ac_details};
use crate::kernel::layout_for_repo;

pub struct AcSuggestScenariosArgs {
    pub ac_id: String,
}

pub fn run(args: AcSuggestScenariosArgs) -> Result<()> {
    let layout = layout_for_repo();

    let ac = get_ac_details(&layout.ledger, &args.ac_id)
        .with_context(|| format!("Failed to look up AC: {}", args.ac_id))?
        .ok_or_else(|| anyhow::anyhow!("AC '{}' not found in spec_ledger.yaml", args.ac_id))?;

    print_scenario_suggestion(&ac);

    Ok(())
}

fn print_scenario_suggestion(ac: &AcDetails) {
    let kernel_indicator = if ac.must_have_ac { " 🔒" } else { "" };

    println!("{}", "═".repeat(80).blue());
    println!("{} {}{}", "AC:".blue().bold(), ac.id.green().bold(), kernel_indicator);
    println!("{} {} ({})", "REQ:".blue(), ac.req_id, ac.req_title);
    println!("{} {}", "Story:".blue(), ac.story_id);
    println!("{}", "═".repeat(80).blue());
    println!();
    println!("{}", "AC Text:".blue().bold());
    println!("  {}", ac.text);
    println!();
    println!("{}", "═".repeat(80).blue());
    println!("{}", "Suggested Gherkin Scenario:".yellow().bold());
    println!("{}", "═".repeat(80).blue());
    println!();

    // Generate the scenario
    let scenario = generate_scenario(ac);
    println!("{}", scenario);

    println!();
    println!("{}", "─".repeat(80).dimmed());
    println!("{}", "Next steps:".yellow().bold());
    println!("  1. Copy the scenario above to the appropriate feature file");
    println!("  2. Customize the Given/When/Then steps to match your domain");
    println!("  3. Run `cargo xtask bdd` to validate Gherkin syntax");
    println!("  4. Implement step definitions if needed");
    println!();
    println!(
        "{}",
        format!("  Suggested file: specs/features/{}.feature", suggest_feature_file(ac)).dimmed()
    );
}

fn generate_scenario(ac: &AcDetails) -> String {
    let scenario_title = extract_scenario_title(&ac.text);
    let (given, when, then) = extract_gherkin_steps(&ac.text);

    format!(
        r#"  @{}
  Scenario: {}
    Given {}
    When {}
    Then {}"#,
        ac.id, scenario_title, given, when, then
    )
}

/// Extract a concise scenario title from the AC text.
fn extract_scenario_title(text: &str) -> String {
    // Clean up the text: remove backticks, normalize whitespace
    let clean =
        text.replace('`', "").replace('\n', " ").split_whitespace().collect::<Vec<_>>().join(" ");

    // Truncate if too long (scenarios should have short titles)
    if clean.len() > 80 { format!("{}...", &clean[..77]) } else { clean }
}

/// Extract Given/When/Then steps from AC text.
///
/// Uses heuristics to detect common patterns in AC descriptions:
/// - Commands/endpoints → When clause
/// - Conditions/states → Given clause
/// - Results/returns → Then clause
fn extract_gherkin_steps(text: &str) -> (String, String, String) {
    let clean = text.replace('\n', " ").trim().to_string();

    // Try to detect patterns in the AC text
    // Pattern 1: "X returns Y when Z" or "X returns Y"
    if let Some(returns_idx) = clean.find(" returns ") {
        let before_returns = &clean[..returns_idx];
        let after_returns = &clean[returns_idx + 9..];

        let when_part = extract_action(before_returns);
        let then_part = format!("the response should {}", after_returns.trim());
        let given_part = "the service is running".to_string();

        return (given_part, when_part, then_part);
    }

    // Pattern 2: "X validates/checks Y" (doctor-style commands)
    if clean.contains(" validates ") || clean.contains(" checks ") {
        let action = extract_command_from_text(&clean);
        let given_part = "the development environment is set up".to_string();
        let when_part = format!("I run {}", action);
        let then_part = "the command should complete successfully".to_string();

        return (given_part, when_part, then_part);
    }

    // Pattern 3: "X runs Y" (command execution)
    if clean.contains(" runs ") {
        let action = extract_command_from_text(&clean);
        let given_part = "the project is configured correctly".to_string();
        let when_part = format!("I run {}", action);
        let then_part = "the command should succeed".to_string();

        return (given_part, when_part, then_part);
    }

    // Pattern 4: "X creates/generates Y" (scaffolding commands)
    if clean.contains(" creates ") || clean.contains(" generates ") {
        let action = extract_command_from_text(&clean);
        let given_part = "the project structure exists".to_string();
        let when_part = format!("I run {}", action);
        let then_part = "the expected artifact should be created".to_string();

        return (given_part, when_part, then_part);
    }

    // Default fallback: use AC text structure
    let given_part = "the system is in the expected state".to_string();
    let when_part = "I perform the action described in this AC".to_string();
    let then_part = clean;

    (given_part, when_part, then_part)
}

/// Extract a command invocation from AC text (e.g., `cargo xtask doctor`).
fn extract_command_from_text(text: &str) -> String {
    // Look for backtick-quoted commands
    if let Some(start) = text.find('`')
        && let Some(end) = text[start + 1..].find('`')
    {
        return format!("\"{}\"", &text[start + 1..start + 1 + end]);
    }

    // Fallback: look for "cargo xtask" pattern
    if let Some(idx) = text.find("cargo xtask") {
        let rest = &text[idx..];
        let end = rest
            .find(|c: char| c == ' ' && rest[..rest.find(c).unwrap_or(0)].matches(' ').count() >= 2)
            .unwrap_or(rest.len().min(40));
        return format!("\"{}\"", &rest[..end].trim());
    }

    "\"the command\"".to_string()
}

/// Extract the action/subject from text before "returns".
fn extract_action(text: &str) -> String {
    // Look for GET/POST/etc. patterns
    let upper = text.to_uppercase();
    if upper.contains("GET ")
        && let Some(idx) = upper.find("GET ")
    {
        let rest = &text[idx..];
        let endpoint = rest.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        return format!("I send a {}", endpoint);
    }

    // Look for command patterns
    if text.contains("cargo xtask") || text.contains('`') {
        let cmd = extract_command_from_text(text);
        return format!("I run {}", cmd);
    }

    format!("I perform \"{}\"", text.trim())
}

/// Suggest an appropriate feature file name based on the AC's requirement.
fn suggest_feature_file(ac: &AcDetails) -> String {
    // Map common requirement patterns to feature files
    let req_lower = ac.req_id.to_lowercase();

    if req_lower.contains("health") {
        return "template_core".to_string();
    }
    if req_lower.contains("version") {
        return "template_core".to_string();
    }
    if req_lower.contains("metrics") {
        return "metrics".to_string();
    }
    if req_lower.contains("platform") || req_lower.contains("plt") {
        return "xtask_devex".to_string();
    }
    if req_lower.contains("security") || req_lower.contains("audit") {
        return "security".to_string();
    }
    if req_lower.contains("graph") {
        return "governance_graph".to_string();
    }
    if req_lower.contains("skill") {
        return "skills_governance".to_string();
    }
    if req_lower.contains("agent") {
        return "agents_governance".to_string();
    }

    // Default: derive from requirement ID
    ac.req_id.to_lowercase().replace("req-", "").replace("-", "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scenario_title_short() {
        let text = "GET /health returns 200 with status 'ok'";
        let title = extract_scenario_title(text);
        assert_eq!(title, "GET /health returns 200 with status 'ok'");
    }

    #[test]
    fn test_extract_scenario_title_with_backticks() {
        let text = "`cargo xtask doctor` validates Rust, Nix, conftest, git";
        let title = extract_scenario_title(text);
        assert_eq!(title, "cargo xtask doctor validates Rust, Nix, conftest, git");
    }

    #[test]
    fn test_extract_scenario_title_truncates_long() {
        let text = "This is a very long acceptance criterion text that goes on and on and should be truncated to fit";
        let title = extract_scenario_title(text);
        assert!(title.len() <= 80);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_extract_gherkin_steps_returns_pattern() {
        let text = "GET /health returns 200 with status 'ok' when service is healthy";
        let (given, when, then) = extract_gherkin_steps(text);
        assert!(when.contains("GET /health"));
        assert!(then.contains("200"));
        assert!(!given.is_empty());
    }

    #[test]
    fn test_extract_gherkin_steps_validates_pattern() {
        let text = "`cargo xtask doctor` validates Rust, Nix, conftest, git";
        let (given, when, then) = extract_gherkin_steps(text);
        assert!(when.contains("cargo xtask doctor"));
        assert!(!given.is_empty());
        assert!(!then.is_empty());
    }

    #[test]
    fn test_extract_command_from_text_backticks() {
        let text = "When `cargo xtask check` is run, it should succeed";
        let cmd = extract_command_from_text(text);
        assert_eq!(cmd, "\"cargo xtask check\"");
    }

    #[test]
    fn test_suggest_feature_file_platform() {
        let ac = AcDetails {
            id: "AC-PLT-001".to_string(),
            text: "test".to_string(),
            story_id: "US-PLT-001".to_string(),
            req_id: "REQ-PLT-ONBOARDING".to_string(),
            req_title: "Onboarding".to_string(),
            must_have_ac: true,
        };
        assert_eq!(suggest_feature_file(&ac), "xtask_devex");
    }

    #[test]
    fn test_suggest_feature_file_health() {
        let ac = AcDetails {
            id: "AC-TPL-001".to_string(),
            text: "test".to_string(),
            story_id: "US-TPL-001".to_string(),
            req_id: "REQ-TPL-HEALTH".to_string(),
            req_title: "Health".to_string(),
            must_have_ac: true,
        };
        assert_eq!(suggest_feature_file(&ac), "template_core");
    }
}
