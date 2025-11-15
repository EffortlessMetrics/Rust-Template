use anyhow::{Context, Result};
use colored::Colorize;
use quick_xml::Reader;
use quick_xml::events::Event;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct AcStatusArgs {
    pub ledger: PathBuf,
    pub features_dir: PathBuf,
    pub junit: PathBuf,
    pub json_report: Option<PathBuf>,
    pub output: PathBuf,
}

impl Default for AcStatusArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            features_dir: PathBuf::from("specs/features"),
            junit: PathBuf::from("target/junit/acceptance.xml"),
            json_report: Some(PathBuf::from("target/ac_report.json")),
            output: PathBuf::from("docs/feature_status.md"),
        }
    }
}

#[derive(Debug, Clone)]
struct Ac {
    id: String,
    story_id: String,
    req_id: String,
    text: String,
    status: AcStatus,
    scenarios: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum AcStatus {
    Pass,
    Fail,
    Unknown,
}

impl AcStatus {
    fn icon(&self) -> &str {
        match self {
            AcStatus::Pass => "✅",
            AcStatus::Fail => "❌",
            AcStatus::Unknown => "❓",
        }
    }

    fn name(&self) -> &str {
        match self {
            AcStatus::Pass => "pass",
            AcStatus::Fail => "fail",
            AcStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
struct Scenario {
    name: String,
    ac_id: String,
    file: String,
}

// Ledger deserialization structures
#[derive(Debug, Deserialize)]
struct Ledger {
    stories: Vec<Story>,
}

#[derive(Debug, Deserialize)]
struct Story {
    id: String,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
struct Requirement {
    id: String,
    acceptance_criteria: Vec<AcceptanceCriteria>,
}

#[derive(Debug, Deserialize)]
struct AcceptanceCriteria {
    id: String,
    text: String,
}

// Cucumber JSON format structures
#[derive(Debug, Deserialize)]
struct CucumberReport(Vec<CucumberFeature>);

#[derive(Debug, Deserialize)]
struct CucumberFeature {
    #[allow(dead_code)]
    name: String,
    uri: String,
    elements: Vec<CucumberElement>,
}

#[derive(Debug, Deserialize)]
struct CucumberElement {
    name: String,
    #[serde(rename = "type")]
    element_type: String,
    tags: Vec<CucumberTag>,
    #[allow(dead_code)]
    line: Option<u32>,
    steps: Vec<CucumberStep>,
}

#[derive(Debug, Deserialize)]
struct CucumberTag {
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    line: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CucumberStep {
    #[allow(dead_code)]
    keyword: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    line: Option<u32>,
    result: CucumberStepResult,
}

#[derive(Debug, Deserialize)]
struct CucumberStepResult {
    status: String,
    #[serde(default)]
    #[allow(dead_code)]
    duration: Option<u64>, // nanoseconds
    #[serde(default)]
    #[allow(dead_code)]
    error_message: Option<String>,
}

pub fn run(args: AcStatusArgs) -> Result<()> {
    // Validate inputs
    if !args.ledger.exists() {
        anyhow::bail!("Ledger file not found: {}", args.ledger.display());
    }
    if !args.features_dir.exists() {
        anyhow::bail!("Features directory not found: {}", args.features_dir.display());
    }

    println!("Parsing ledger: {}", args.ledger.display());
    let mut acs = parse_ledger(&args.ledger)?;
    println!("  Found {} ACs", acs.len());

    // PRIMARY PATH: Structured JSON report from acceptance tests
    // The Cucumber JSON format provides all necessary metadata (tags, status, etc.)
    // in a single structured file, eliminating the need to parse Gherkin text.
    //
    // FALLBACK PATH: JUnit XML + feature file parsing
    // This is a legacy path for backward compatibility and may be removed in a future major version.
    // The JUnit path requires fragile text parsing and string matching.
    let (scenarios, ac_results) = if let Some(json_path) = &args.json_report {
        if json_path.exists() {
            println!("Parsing JSON report: {}", json_path.display());
            let (scens, results) = parse_cucumber_json(json_path)?;
            println!("  Found {} scenarios", scens.len());
            println!("  Found results for {} ACs", results.len());
            (scens, results)
        } else {
            println!("JSON report not found: {}", json_path.display());
            println!("Falling back to JUnit + feature parsing (legacy)");
            fallback_to_junit(&args)?
        }
    } else {
        println!("JSON report disabled, using JUnit + feature parsing (legacy)");
        fallback_to_junit(&args)?
    };

    println!("Generating status: {}", args.output.display());
    generate_status_md(&mut acs, &scenarios, &ac_results, &args.output)?;

    // Check for failures
    let failed: Vec<_> =
        acs.values().filter(|ac| ac.status == AcStatus::Fail).map(|ac| ac.id.as_str()).collect();

    if !failed.is_empty() {
        eprintln!("\n{} {} AC(s) failed: {}", "❌".red(), failed.len(), failed.join(", "));
        anyhow::bail!("One or more ACs failed");
    }

    println!("\n{} All ACs passed", "✓".green());
    Ok(())
}

fn fallback_to_junit(
    args: &AcStatusArgs,
) -> Result<(HashMap<String, Scenario>, HashMap<String, AcStatus>)> {
    if !args.junit.exists() {
        anyhow::bail!(
            "JUnit XML not found: {}\nRun acceptance tests first: cargo test -p acceptance",
            args.junit.display()
        );
    }

    println!("Parsing features: {}", args.features_dir.display());
    let scenarios = parse_features(&args.features_dir)?;
    println!("  Found {} scenarios", scenarios.len());

    println!("Parsing JUnit results: {}", args.junit.display());
    let ac_results = parse_junit(&args.junit, &scenarios)?;
    println!("  Found results for {} ACs", ac_results.len());

    Ok((scenarios, ac_results))
}

fn parse_cucumber_json(
    json_path: &Path,
) -> Result<(HashMap<String, Scenario>, HashMap<String, AcStatus>)> {
    let content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON report: {}", json_path.display()))?;

    let report: CucumberReport = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON report: {}", json_path.display()))?;

    let mut scenarios: HashMap<String, Scenario> = HashMap::new();
    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();
    // Note: Cucumber JSON doesn't include @ in tag names
    let ac_pattern = Regex::new(r"^(AC-[A-Z0-9-]+)$")?;

    for feature in report.0 {
        for element in feature.elements {
            // Only process scenarios (not hooks or backgrounds)
            if element.element_type == "scenario" {
                // Extract AC IDs from tags
                let ac_ids: Vec<String> = element
                    .tags
                    .iter()
                    .filter_map(|tag| {
                        ac_pattern.captures(&tag.name).map(|caps| caps[1].to_string())
                    })
                    .collect();

                // Determine if scenario passed (all steps passed)
                let passed = element.steps.iter().all(|step| step.result.status == "passed");

                // Record result for each AC ID
                for ac_id in &ac_ids {
                    ac_results.entry(ac_id.clone()).or_default().push(passed);

                    // Store scenario information (using first AC ID)
                    scenarios.insert(
                        element.name.clone(),
                        Scenario {
                            name: element.name.clone(),
                            ac_id: ac_id.clone(),
                            file: feature.uri.clone(),
                        },
                    );
                }
            }
        }
    }

    // Aggregate: AC passes only if all scenarios pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status =
            if results.iter().all(|&passed| passed) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok((scenarios, ac_status))
}

fn parse_ledger(ledger_path: &Path) -> Result<HashMap<String, Ac>> {
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read ledger: {}", ledger_path.display()))?;

    let ledger: Ledger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse ledger YAML: {}", ledger_path.display()))?;

    let mut acs = HashMap::new();

    for story in ledger.stories {
        for req in story.requirements {
            for ac in req.acceptance_criteria {
                acs.insert(
                    ac.id.clone(),
                    Ac {
                        id: ac.id.clone(),
                        story_id: story.id.clone(),
                        req_id: req.id.clone(),
                        text: ac.text,
                        status: AcStatus::Unknown,
                        scenarios: Vec::new(),
                    },
                );
            }
        }
    }

    Ok(acs)
}

fn parse_features(features_dir: &Path) -> Result<HashMap<String, Scenario>> {
    let mut scenarios = HashMap::new();

    let ac_pattern = Regex::new(r"@(AC-[A-Z0-9-]+)")?;
    let scenario_pattern = Regex::new(r"^\s*Scenario(?:\s+Outline)?:\s+(.+)")?;
    let tag_pattern = Regex::new(r"@[\w-]+")?;

    for entry in WalkDir::new(features_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "feature"))
    {
        let content = fs::read_to_string(entry.path())?;
        let relative_path = entry
            .path()
            .strip_prefix(features_dir.parent().unwrap_or(features_dir))
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        let mut current_tags: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim_start();

            // Collect tags from lines starting with @
            if trimmed.starts_with('@') {
                for tag_match in tag_pattern.find_iter(trimmed) {
                    current_tags.push(tag_match.as_str().to_string());
                }
            }
            // Match scenario and attach accumulated tags
            else if let Some(caps) = scenario_pattern.captures(trimmed) {
                let scenario_name = caps[1].trim();

                // Look for AC ID in collected tags
                let ac_id = current_tags
                    .iter()
                    .find_map(|tag| ac_pattern.captures(tag))
                    .map(|caps| caps[1].to_string());

                if let Some(ac_id) = ac_id {
                    scenarios.insert(
                        scenario_name.to_string(),
                        Scenario {
                            name: scenario_name.to_string(),
                            ac_id,
                            file: relative_path.clone(),
                        },
                    );
                }

                // Reset tags after processing scenario
                current_tags.clear();
            }
            // Reset tags on other Gherkin keywords (but not on blank lines or comments)
            else if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && (trimmed.starts_with("Feature:")
                    || trimmed.starts_with("Background:")
                    || trimmed.starts_with("Rule:")
                    || trimmed.starts_with("Examples:")
                    || trimmed.starts_with("Given ")
                    || trimmed.starts_with("When ")
                    || trimmed.starts_with("Then ")
                    || trimmed.starts_with("And ")
                    || trimmed.starts_with("But ")
                    || trimmed.starts_with("|"))
            {
                current_tags.clear();
            }
        }
    }

    Ok(scenarios)
}

fn normalize_testcase_name(name: &str) -> String {
    // Extract scenario name from JUnit testcase name
    // Format: "Scenario: <name>: <file>:<line>:<col>"
    let scenario_pattern = Regex::new(r"Scenario:\s+(.+?):\s+").unwrap();

    if let Some(caps) = scenario_pattern.captures(name) {
        let mut scenario_name = caps[1].trim().to_string();
        // Remove example/row suffixes
        let suffix_pattern = Regex::new(r"\s*\((?:row|example)\s+\d+\)\s*$").unwrap();
        scenario_name = suffix_pattern.replace(&scenario_name, "").to_string();
        return scenario_name;
    }

    name.to_string()
}

fn parse_junit(
    junit_path: &Path,
    scenarios: &HashMap<String, Scenario>,
) -> Result<HashMap<String, AcStatus>> {
    let content = fs::read_to_string(junit_path)?;
    let mut reader = Reader::from_str(&content);

    let mut ac_results: HashMap<String, Vec<bool>> = HashMap::new();
    let mut buf = Vec::new();

    let mut current_testcase: Option<String> = None;
    let mut testcase_passed = true;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"testcase" => {
                        // Extract testcase name
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                let name = String::from_utf8_lossy(&attr.value).to_string();
                                current_testcase = Some(name);
                                testcase_passed = true;
                            }
                        }
                    }
                    b"failure" | b"error" => {
                        testcase_passed = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"testcase"
                    && let Some(tc_name) = current_testcase.take()
                {
                    let normalized = normalize_testcase_name(&tc_name);

                    // Find matching scenario
                    if let Some(scenario) = scenarios.get(&normalized) {
                        ac_results.entry(scenario.ac_id.clone()).or_default().push(testcase_passed);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    // Aggregate: AC passes only if all testcases pass
    let mut ac_status = HashMap::new();
    for (ac_id, results) in ac_results {
        let status =
            if results.iter().all(|&passed| passed) { AcStatus::Pass } else { AcStatus::Fail };
        ac_status.insert(ac_id, status);
    }

    Ok(ac_status)
}

fn generate_status_md(
    acs: &mut HashMap<String, Ac>,
    scenarios: &HashMap<String, Scenario>,
    ac_results: &HashMap<String, AcStatus>,
    output_path: &Path,
) -> Result<()> {
    // Map scenarios to ACs
    for scenario in scenarios.values() {
        if let Some(ac) = acs.get_mut(&scenario.ac_id) {
            ac.scenarios.push(scenario.name.clone());
        }
    }

    // Update AC status from results
    for (ac_id, status) in ac_results {
        if let Some(ac) = acs.get_mut(ac_id) {
            ac.status = status.clone();
        }
    }

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut output = String::new();
    output.push_str("# Feature Status\n\n");
    output.push_str("Auto-generated AC status from acceptance tests.\n\n");

    output.push_str("## AC Status Summary\n\n");
    output.push_str("| AC ID | Story | Requirement | Status | Scenarios |\n");
    output.push_str("|-------|-------|-------------|--------|----------|\n");

    let mut ac_ids: Vec<_> = acs.keys().cloned().collect();
    ac_ids.sort();

    for ac_id in &ac_ids {
        let ac = &acs[ac_id];
        output.push_str(&format!(
            "| {} | {} | {} | {} {} | {} |\n",
            ac.id,
            ac.story_id,
            ac.req_id,
            ac.status.icon(),
            ac.status.name(),
            ac.scenarios.len()
        ));
    }

    // Unmapped ACs
    let unmapped: Vec<_> = acs.values().filter(|ac| ac.scenarios.is_empty()).collect();
    if !unmapped.is_empty() {
        output.push_str("\n## Unmapped ACs\n\n");
        output.push_str("ACs with no mapped scenarios:\n\n");
        for ac in unmapped {
            let text_preview = if ac.text.len() > 100 {
                // Safe truncation at character boundary
                let mut end = 100.min(ac.text.len());
                while end > 0 && !ac.text.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &ac.text[..end])
            } else {
                ac.text.clone()
            };
            output.push_str(&format!("- {}: {}\n", ac.id, text_preview));
        }
    }

    // Unmapped scenarios
    let unmapped_scenarios: Vec<_> =
        scenarios.values().filter(|s| !acs.contains_key(&s.ac_id)).collect();
    if !unmapped_scenarios.is_empty() {
        output.push_str("\n## Unmapped Scenarios\n\n");
        output.push_str("Scenarios referencing non-existent ACs:\n\n");
        for scenario in unmapped_scenarios {
            output.push_str(&format!(
                "- Scenario '{}' references {} (in {})\n",
                scenario.name, scenario.ac_id, scenario.file
            ));
        }
    }

    fs::write(output_path, output)?;
    println!("{} Generated {}", "✓".green(), output_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_features_with_tags_on_previous_line() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Test Feature
  As a user
  I want to test tag parsing
  So that ACs are correctly mapped

  @AC-TEST-001 @smoke
  Scenario: First scenario
    When I do something
    Then it works

  @AC-TEST-002
  Scenario: Second scenario
    When I do something else
    Then it also works
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 2);
        assert!(scenarios.contains_key("First scenario"));
        assert!(scenarios.contains_key("Second scenario"));

        let scenario1 = &scenarios["First scenario"];
        assert_eq!(scenario1.ac_id, "AC-TEST-001");

        let scenario2 = &scenarios["Second scenario"];
        assert_eq!(scenario2.ac_id, "AC-TEST-002");
    }

    #[test]
    fn test_parse_features_with_multiple_tags_on_same_line() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Multi-tag Test

  @AC-TEST-003 @wip @integration
  Scenario: Scenario with multiple tags
    Given something
    When action
    Then result
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 1);
        let scenario = &scenarios["Scenario with multiple tags"];
        assert_eq!(scenario.ac_id, "AC-TEST-003");
    }

    #[test]
    fn test_parse_features_ignores_scenarios_without_ac_tags() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Mixed scenarios

  @smoke
  Scenario: No AC tag
    When something happens
    Then no mapping

  @AC-TEST-004
  Scenario: Has AC tag
    When something else happens
    Then mapped correctly
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        // Only the scenario with AC tag should be included
        assert_eq!(scenarios.len(), 1);
        assert!(scenarios.contains_key("Has AC tag"));
        assert!(!scenarios.contains_key("No AC tag"));
    }

    #[test]
    fn test_parse_features_scenario_outline() {
        let temp_dir = TempDir::new().unwrap();
        let features_dir = temp_dir.path().join("features");
        fs::create_dir_all(&features_dir).unwrap();

        let feature_content = r#"Feature: Outline test

  @AC-TEST-005
  Scenario Outline: Parameterized scenario
    When I use <value>
    Then I get <result>

    Examples:
      | value | result |
      | 1     | one    |
      | 2     | two    |
"#;

        fs::write(features_dir.join("test.feature"), feature_content).unwrap();

        let scenarios = parse_features(&features_dir).unwrap();

        assert_eq!(scenarios.len(), 1);
        let scenario = &scenarios["Parameterized scenario"];
        assert_eq!(scenario.ac_id, "AC-TEST-005");
    }
}
