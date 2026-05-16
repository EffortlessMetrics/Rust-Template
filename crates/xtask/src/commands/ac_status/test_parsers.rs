// Imports for test-only functions
use ac_kernel::AcStatus;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::super::ac_parsing::Scenario;
#[cfg(test)]
use super::super::ac_parsing::{
    AC_PATTERN_WITH_AT, SCENARIO_PATTERN, TAG_PATTERN, TESTCASE_SCENARIO_PATTERN,
    TESTCASE_SUFFIX_PATTERN,
};
#[cfg(test)]
use quick_xml::Reader;
#[cfg(test)]
use quick_xml::events::Event;
#[cfg(test)]
use walkdir::WalkDir;

/// Parse feature files and extract scenarios with their AC tags.
///
/// Returns full Scenario structs (including file path). Used by tests.
/// For production code, use `ac_parsing::parse_features` which returns simpler mapping.
#[cfg(test)]
pub(super) fn parse_features(features_dir: &Path) -> Result<HashMap<String, Scenario>> {
    let mut scenarios = HashMap::new();

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
                for tag_match in TAG_PATTERN.find_iter(trimmed) {
                    current_tags.push(tag_match.as_str().to_string());
                }
            }
            // Match scenario and attach accumulated tags
            else if let Some(caps) = SCENARIO_PATTERN.captures(trimmed) {
                let scenario_name = caps[1].trim();

                // Look for AC ID in collected tags
                let ac_id = current_tags
                    .iter()
                    .find_map(|tag| AC_PATTERN_WITH_AT.captures(tag))
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

/// Normalize JUnit testcase names for scenario matching.
///
/// Used by the JUnit fallback parsing path. The primary parser uses Cucumber JSON.
#[cfg(test)]
#[allow(dead_code)] // Fallback implementation kept for reference; primary path uses ac_parsing
pub(super) fn normalize_testcase_name(name: &str) -> String {
    // Extract scenario name from JUnit testcase name
    // Format: "Scenario: <name>: <file>:<line>:<col>"
    if let Some(caps) = TESTCASE_SCENARIO_PATTERN.captures(name) {
        let scenario_name = caps[1].trim();
        // Remove example/row suffixes
        let normalized = TESTCASE_SUFFIX_PATTERN.replace(scenario_name, "");
        return normalized.to_string();
    }

    name.to_string()
}

/// Parse JUnit XML and extract AC test results using scenario metadata.
///
/// Fallback parser for systems without Cucumber JSON support.
/// The primary path uses `ac_parsing::parse_junit_with_scenarios()`.
#[cfg(test)]
#[allow(dead_code)] // Fallback implementation kept for reference; primary path uses ac_parsing
pub(super) fn parse_junit(
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
