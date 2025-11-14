use anyhow::{Context, Result};
use colored::Colorize;
use quick_xml::events::Event;
use quick_xml::Reader;
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
    pub output: PathBuf,
}

impl Default for AcStatusArgs {
    fn default() -> Self {
        Self {
            ledger: PathBuf::from("specs/spec_ledger.yaml"),
            features_dir: PathBuf::from("specs/features"),
            junit: PathBuf::from("target/junit/acceptance.xml"),
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

pub fn run(args: AcStatusArgs) -> Result<()> {
    // Validate inputs
    if !args.ledger.exists() {
        anyhow::bail!("Ledger file not found: {}", args.ledger.display());
    }
    if !args.features_dir.exists() {
        anyhow::bail!("Features directory not found: {}", args.features_dir.display());
    }
    if !args.junit.exists() {
        anyhow::bail!(
            "JUnit XML not found: {}\nRun acceptance tests first: cargo test -p acceptance",
            args.junit.display()
        );
    }

    println!("Parsing ledger: {}", args.ledger.display());
    let mut acs = parse_ledger(&args.ledger)?;
    println!("  Found {} ACs", acs.len());

    println!("Parsing features: {}", args.features_dir.display());
    let scenarios = parse_features(&args.features_dir)?;
    println!("  Found {} scenarios", scenarios.len());

    println!("Parsing JUnit results: {}", args.junit.display());
    let ac_results = parse_junit(&args.junit, &scenarios)?;
    println!("  Found results for {} ACs", ac_results.len());

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

    // Regex to match tags followed by Scenario/Scenario Outline
    let pattern = Regex::new(r"(@[\w-]+(?:\s+@[\w-]+)*)\s+Scenario(?:\s+Outline)?:\s+(.+)")?;
    let ac_pattern = Regex::new(r"@(AC-[A-Z0-9-]+)")?;

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

        for caps in pattern.captures_iter(&content) {
            let tags_str = &caps[1];
            let scenario_name = caps[2].trim();

            // Extract AC ID from tags
            if let Some(ac_match) = ac_pattern.captures(tags_str) {
                let ac_id = ac_match[1].to_string();
                scenarios.insert(
                    scenario_name.to_string(),
                    Scenario {
                        name: scenario_name.to_string(),
                        ac_id,
                        file: relative_path.clone(),
                    },
                );
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
                if e.name().as_ref() == b"testcase" {
                    if let Some(tc_name) = current_testcase.take() {
                        let normalized = normalize_testcase_name(&tc_name);

                        // Find matching scenario
                        if let Some(scenario) = scenarios.get(&normalized) {
                            ac_results
                                .entry(scenario.ac_id.clone())
                                .or_default()
                                .push(testcase_passed);
                        }
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
                format!("{}...", &ac.text[..100])
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
