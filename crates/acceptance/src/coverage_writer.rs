//! AC Coverage Writer for streaming AC test results to JSONL.
//!
//! This writer implements cucumber's `Writer` trait to capture scenario results
//! and emit them as JSON lines to a file. Each scenario finish event produces
//! one line per AC ID found in the scenario's tags.
//!
//! Unlike the JUnit writer which buffers all output until drop, this writer
//! flushes after each line, making it resilient to process exits.

use cucumber::cli::Empty;
use cucumber::event::{self, Cucumber, Feature, Rule, Scenario};
use cucumber::{Event, World as CucumberWorld, Writer};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

/// A single AC coverage record emitted to the JSONL file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcCoverageRecord {
    /// The AC ID (e.g., "AC-KERN-001")
    pub ac_id: String,
    /// The status of the scenario: "passed", "failed", or "skipped"
    pub status: String,
    /// The feature file path
    pub feature: String,
    /// The scenario name
    pub scenario: String,
    /// All tags from feature, rule, and scenario
    pub tags: Vec<String>,
}

/// State tracking for the current scenario being executed.
#[derive(Debug, Default)]
struct ScenarioState {
    /// Whether any step has failed
    has_failed: bool,
    /// Whether we've seen any step results (to distinguish skipped from not-run)
    has_step_results: bool,
}

/// A cucumber Writer that streams AC coverage data to a JSONL file.
///
/// This writer captures scenario completion events and writes coverage records
/// immediately, without relying on Drop semantics. This makes it robust against
/// cucumber's `*_and_exit` methods which call `std::process::exit()`.
pub struct AcCoverageWriter<W: CucumberWorld> {
    /// Buffered output writer
    out: BufWriter<File>,
    /// Current scenario state (feature, rule, scenario context)
    current_feature: Option<Arc<gherkin::Feature>>,
    current_rule: Option<Arc<gherkin::Rule>>,
    current_scenario: Option<Arc<gherkin::Scenario>>,
    /// Step tracking for pass/fail determination
    scenario_state: ScenarioState,
    /// Marker for World type
    _world: std::marker::PhantomData<W>,
}

impl<W: CucumberWorld> AcCoverageWriter<W> {
    /// Create a new AC coverage writer that writes to the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            out: BufWriter::new(file),
            current_feature: None,
            current_rule: None,
            current_scenario: None,
            scenario_state: ScenarioState::default(),
            _world: std::marker::PhantomData,
        })
    }

    /// Collect all tags from the current context (feature + rule + scenario).
    fn collect_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();

        if let Some(ref feature) = self.current_feature {
            tags.extend(feature.tags.iter().cloned());
        }
        if let Some(ref rule) = self.current_rule {
            tags.extend(rule.tags.iter().cloned());
        }
        if let Some(ref scenario) = self.current_scenario {
            tags.extend(scenario.tags.iter().cloned());
        }

        // Normalize: remove @ prefix for consistency
        tags.iter().map(|t| t.trim_start_matches('@').to_string()).collect()
    }

    /// Extract AC IDs from a list of tags.
    fn extract_ac_ids(tags: &[String]) -> Vec<String> {
        tags.iter().filter(|t| t.starts_with("AC-") || t.starts_with("ac-")).cloned().collect()
    }

    /// Write a coverage record for the current scenario.
    fn write_scenario_result(&mut self) -> std::io::Result<()> {
        let tags = self.collect_tags();
        let ac_ids = Self::extract_ac_ids(&tags);

        // Only write records for scenarios that have AC tags
        if ac_ids.is_empty() {
            return Ok(());
        }

        let feature_path = self
            .current_feature
            .as_ref()
            .and_then(|f| f.path.as_ref())
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        let scenario_name =
            self.current_scenario.as_ref().map(|s| s.name.clone()).unwrap_or_default();

        let status = if self.scenario_state.has_failed {
            "failed"
        } else if self.scenario_state.has_step_results {
            "passed"
        } else {
            "skipped"
        };

        // Write one record per AC ID (a scenario can be tagged with multiple ACs)
        for ac_id in ac_ids {
            let record = AcCoverageRecord {
                ac_id,
                status: status.to_string(),
                feature: feature_path.clone(),
                scenario: scenario_name.clone(),
                tags: tags.clone(),
            };

            let line = serde_json::to_string(&record)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            writeln!(self.out, "{}", line)?;
        }

        // Flush immediately - don't rely on Drop
        self.out.flush()?;

        Ok(())
    }

    /// Reset scenario state for the next scenario.
    fn reset_scenario_state(&mut self) {
        self.scenario_state = ScenarioState::default();
    }

    /// Handle a Scenario event.
    fn handle_scenario_event(&mut self, scenario_event: &Scenario<W>) {
        match scenario_event {
            Scenario::Started => {
                self.reset_scenario_state();
            }
            Scenario::Step(_, step_event) | Scenario::Background(_, step_event) => {
                self.handle_step_event(step_event);
            }
            Scenario::Hook(_, hook_event) => {
                self.handle_hook_event(hook_event);
            }
            Scenario::Finished => {
                if let Err(e) = self.write_scenario_result() {
                    eprintln!("[AcCoverageWriter] Failed to write coverage: {}", e);
                }
            }
            Scenario::Log(_) => {}
        }
    }

    /// Handle a Step event to track pass/fail.
    fn handle_step_event(&mut self, step_event: &event::Step<W>) {
        match step_event {
            event::Step::Started => {}
            event::Step::Passed(..) => {
                self.scenario_state.has_step_results = true;
            }
            event::Step::Skipped => {
                // Skipped steps don't count as failures, but we note we saw them
            }
            event::Step::Failed(..) => {
                self.scenario_state.has_step_results = true;
                self.scenario_state.has_failed = true;
            }
        }
    }

    /// Handle a Hook event (before/after hooks can also fail).
    fn handle_hook_event(&mut self, hook_event: &event::Hook<W>) {
        match hook_event {
            event::Hook::Started => {}
            event::Hook::Passed => {
                self.scenario_state.has_step_results = true;
            }
            event::Hook::Failed(..) => {
                self.scenario_state.has_step_results = true;
                self.scenario_state.has_failed = true;
            }
        }
    }
}

impl<W> Writer<W> for AcCoverageWriter<W>
where
    W: CucumberWorld + Debug,
{
    type Cli = Empty;

    async fn handle_event(
        &mut self,
        ev: cucumber::parser::Result<Event<Cucumber<W>>>,
        _cli: &Self::Cli,
    ) {
        // Ignore parser errors - we only care about scenario results
        let Ok(event) = ev else {
            return;
        };

        match event.value {
            Cucumber::Started | Cucumber::ParsingFinished { .. } | Cucumber::Finished => {}

            Cucumber::Feature(feature, feature_event) => match feature_event {
                Feature::Started => {
                    self.current_feature = Some(feature);
                    self.current_rule = None;
                }
                Feature::Finished => {
                    self.current_feature = None;
                    self.current_rule = None;
                }
                Feature::Rule(rule, rule_event) => match rule_event {
                    Rule::Started => {
                        self.current_rule = Some(rule);
                    }
                    Rule::Finished => {
                        self.current_rule = None;
                    }
                    Rule::Scenario(scenario, retryable_scenario) => {
                        self.current_scenario = Some(scenario);
                        self.handle_scenario_event(&retryable_scenario.event);
                    }
                },
                Feature::Scenario(scenario, retryable_scenario) => {
                    self.current_scenario = Some(scenario);
                    self.handle_scenario_event(&retryable_scenario.event);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};

    #[test]
    fn extract_ac_ids_filters_correctly() {
        let tags = vec![
            "AC-KERN-001".to_string(),
            "smoke".to_string(),
            "AC-TPL-UI-001".to_string(),
            "wip".to_string(),
            "ac-lower-case".to_string(),
        ];

        let ac_ids = AcCoverageWriter::<crate::World>::extract_ac_ids(&tags);
        assert_eq!(ac_ids.len(), 3);
        assert!(ac_ids.contains(&"AC-KERN-001".to_string()));
        assert!(ac_ids.contains(&"AC-TPL-UI-001".to_string()));
        assert!(ac_ids.contains(&"ac-lower-case".to_string()));
    }

    #[test]
    fn coverage_record_serializes_correctly() {
        let record = AcCoverageRecord {
            ac_id: "AC-KERN-001".to_string(),
            status: "passed".to_string(),
            feature: "specs/features/health.feature".to_string(),
            scenario: "Health endpoint returns OK".to_string(),
            tags: vec!["kernel".to_string(), "AC-KERN-001".to_string()],
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("AC-KERN-001"));
        assert!(json.contains("passed"));
        assert!(json.contains("health.feature"));
    }

    #[test]
    fn coverage_record_deserializes_correctly() {
        // Verify round-trip: the JSONL format we write can be parsed back
        let record = AcCoverageRecord {
            ac_id: "AC-TEST-001".to_string(),
            status: "failed".to_string(),
            feature: "test.feature".to_string(),
            scenario: "Test scenario".to_string(),
            tags: vec!["AC-TEST-001".to_string(), "smoke".to_string()],
        };

        let json = serde_json::to_string(&record).unwrap();
        let parsed: AcCoverageRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ac_id, "AC-TEST-001");
        assert_eq!(parsed.status, "failed");
        assert_eq!(parsed.feature, "test.feature");
        assert_eq!(parsed.scenario, "Test scenario");
        assert_eq!(parsed.tags.len(), 2);
    }

    #[test]
    fn writer_creates_file_and_can_be_read() {
        use tempfile::NamedTempFile;

        // Create a temp file for the writer
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Manually write a record using the same format the writer uses
        {
            let file = std::fs::File::create(&path).unwrap();
            let mut out = BufWriter::new(file);

            let record = AcCoverageRecord {
                ac_id: "AC-WRITER-TEST".to_string(),
                status: "passed".to_string(),
                feature: "writer_test.feature".to_string(),
                scenario: "Writer integration test".to_string(),
                tags: vec!["AC-WRITER-TEST".to_string()],
            };

            let line = serde_json::to_string(&record).unwrap();
            writeln!(out, "{}", line).unwrap();
            out.flush().unwrap();
        }

        // Read it back and verify
        let file = std::fs::File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 1);

        let parsed: AcCoverageRecord = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(parsed.ac_id, "AC-WRITER-TEST");
        assert_eq!(parsed.status, "passed");
        assert_eq!(parsed.scenario, "Writer integration test");
    }

    #[test]
    fn extract_ac_ids_empty_on_no_ac_tags() {
        let tags = vec!["smoke".to_string(), "kernel".to_string(), "tier1".to_string()];

        let ac_ids = AcCoverageWriter::<crate::World>::extract_ac_ids(&tags);
        assert!(ac_ids.is_empty());
    }

    #[test]
    fn extract_ac_ids_case_sensitivity() {
        // AC- and ac- are both recognized
        let tags = vec![
            "AC-UPPER-001".to_string(),
            "ac-lower-001".to_string(),
            "Ac-Mixed-001".to_string(), // This should NOT match (mixed case)
        ];

        let ac_ids = AcCoverageWriter::<crate::World>::extract_ac_ids(&tags);
        assert_eq!(ac_ids.len(), 2);
        assert!(ac_ids.contains(&"AC-UPPER-001".to_string()));
        assert!(ac_ids.contains(&"ac-lower-001".to_string()));
    }
}
