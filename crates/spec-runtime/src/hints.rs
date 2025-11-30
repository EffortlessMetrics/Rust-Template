use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Hint kind (categorization for filtering and display)
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HintKind {
    Task,
    Governance,
    Policy,
    Flow,
}

/// Priority level for hint ordering and agent decision-making
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HintPriority {
    Low,
    Medium,
    High,
}

/// Hint status (reflects the underlying task/work item status)
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HintStatus {
    Open,
    InProgress,
    Done,
}

/// Why the hint is being suggested
#[derive(Debug, Clone, Serialize)]
pub struct HintReason {
    /// Machine-readable code (e.g. "TASK_OPEN", "TASK_SECURITY_CRITICAL")
    pub code: String,
    /// Human-readable rationale
    pub details: String,
}

/// The entity that the hint is about
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HintTarget {
    Task { id: String },
    Flow { id: String },
    Requirement { id: String },
    Ac { id: String },
}

/// Links to related resources (specs, ADRs, docs)
#[derive(Debug, Clone, Serialize, Default)]
pub struct HintLinks {
    /// Reference to spec_ledger.yaml section (e.g. "specs/spec_ledger.yaml#REQ-TPL-001")
    pub spec: Option<String>,
    /// Task ID for convenience
    pub task: Option<String>,
    /// Documentation paths
    pub docs: Vec<String>,
    /// ADR file paths or IDs
    pub adrs: Vec<String>,
    /// Extra arbitrary link slots
    pub extra: BTreeMap<String, String>,
}

/// A complete hint suggesting work to an agent
#[derive(Debug, Clone, Serialize)]
pub struct Hint {
    pub id: String,
    pub kind: HintKind,
    pub title: String,
    pub priority: HintPriority,
    pub status: HintStatus,
    pub reason: HintReason,
    pub target: HintTarget,
    pub tags: Vec<String>,
    pub links: HintLinks,
}

/// AC execution status parsed from feature_status.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AcExecutionStatus {
    Pass { executed: u32, total: u32 },
    Fail { executed: u32, total: u32 },
    Unknown { total: u32 },
}

/// Coverage information for an AC from feature_status.md
#[derive(Debug, Clone, Serialize)]
pub struct AcCoverage {
    pub ac_id: String,
    pub story_id: String,
    pub requirement_id: String,
    pub status: AcExecutionStatus,
}

/// Index from AC ID to coverage (built from feature_status.md)
pub type AcCoverageIndex = std::collections::HashMap<String, AcCoverage>;

/// Hint filter for querying hints
#[derive(Debug, Clone, Default)]
pub struct HintFilter {
    pub kinds: Option<Vec<HintKind>>,
    pub priorities: Option<Vec<HintPriority>>,
    pub status: Option<HintStatus>,
    pub limit: Option<usize>,
}

/// The HintEngine builds prioritized hints from tasks and AC coverage
pub struct HintEngine {
    ac_index: AcCoverageIndex,
    tasks: Vec<crate::Task>,
}

impl HintEngine {
    /// Create a new HintEngine from AC coverage and tasks
    pub fn new(ac_index: AcCoverageIndex, tasks: Vec<crate::Task>) -> Self {
        Self { ac_index, tasks }
    }

    /// Generate hints from open/in-progress tasks
    pub fn task_hints(&self) -> Vec<Hint> {
        self.tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.status.as_str(),
                    "Todo" | "InProgress" | "todo" | "in_progress" | "in-progress"
                )
            })
            .enumerate()
            .map(|(idx, task)| {
                let hint_id = format!("HINT-TASK-{:03}", idx + 1);
                self.build_hint_for_task(&hint_id, task)
            })
            .collect()
    }

    /// Apply a filter to hints
    pub fn apply_filter(&self, hints: Vec<Hint>, filter: &HintFilter) -> Vec<Hint> {
        let mut filtered = hints;

        // Filter by kind
        if let Some(kinds) = &filter.kinds {
            filtered.retain(|h| kinds.contains(&h.kind));
        }

        // Filter by priority
        if let Some(priorities) = &filter.priorities {
            filtered.retain(|h| priorities.contains(&h.priority));
        }

        // Filter by status
        if let Some(status) = filter.status {
            filtered.retain(|h| h.status == status);
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        filtered
    }

    fn build_hint_for_task(&self, hint_id: &str, task: &crate::Task) -> Hint {
        // Determine priority based on labels
        let priority = if task.labels.iter().any(|l| {
            matches!(l.as_str(), "security" | "vuln" | "dependencies" | "governance" | "platform")
        }) {
            HintPriority::High
        } else {
            HintPriority::Medium
        };

        // Build reason based on task and AC status
        let (reason_code, reason_details) = self.build_reason_for_task(task);

        // Collect links
        let spec_link = Some(format!("specs/spec_ledger.yaml#{}", task.requirement));

        let mut links = HintLinks {
            spec: spec_link,
            task: Some(task.id.clone()),
            docs: vec![],
            adrs: vec![],
            extra: BTreeMap::new(),
        };

        // Add security-related ADRs for security tasks
        if task.labels.iter().any(|l| l == "security" || l == "dependencies") {
            links.adrs.push("docs/adr/0006-supply-chain-hardening.md".to_string());
        }

        Hint {
            id: hint_id.to_string(),
            kind: HintKind::Task,
            title: task.title.clone(),
            priority,
            status: match task.status.as_str() {
                "Todo" | "todo" => HintStatus::Open,
                "InProgress" | "in_progress" | "in-progress" => HintStatus::InProgress,
                "Done" | "done" => HintStatus::Done,
                _ => HintStatus::Open,
            },
            reason: HintReason { code: reason_code, details: reason_details },
            target: HintTarget::Task { id: task.id.clone() },
            tags: task.labels.clone(),
            links,
        }
    }

    fn build_reason_for_task(&self, task: &crate::Task) -> (String, String) {
        let mut details = format!(
            "Task {} is {} with {} linked ACs.",
            task.id,
            task.status.to_lowercase(),
            task.acs.len()
        );

        // Check if all ACs are passing
        let all_acs_pass = task.acs.iter().all(|ac_id| {
            self.ac_index
                .get(ac_id)
                .map(|cov| matches!(cov.status, AcExecutionStatus::Pass { .. }))
                .unwrap_or(false)
        });

        let code = if all_acs_pass {
            if task.labels.iter().any(|l| l == "security") {
                "TASK_OPEN_SECURITY_CRITICAL"
            } else if task.labels.iter().any(|l| l == "governance") {
                "TASK_OPEN_GOVERNANCE_READY"
            } else {
                "TASK_OPEN_ACS_PASS"
            }
        } else {
            "TASK_OPEN_ACS_INCOMPLETE"
        };

        // Add AC status information
        let mut ac_notes = Vec::new();
        for ac_id in &task.acs {
            if let Some(coverage) = self.ac_index.get(ac_id) {
                match coverage.status {
                    AcExecutionStatus::Pass { executed, total } => {
                        ac_notes.push(format!("{} passing ({}/{})", ac_id, executed, total));
                    }
                    AcExecutionStatus::Fail { executed, total } => {
                        ac_notes.push(format!("{} failing ({}/{})", ac_id, executed, total));
                    }
                    AcExecutionStatus::Unknown { total } => {
                        ac_notes.push(format!("{} not yet tested ({} total)", ac_id, total));
                    }
                }
            }
        }

        if !ac_notes.is_empty() {
            details.push_str(&format!(" AC status: {}.", ac_notes.join(", ")));
        }

        if task.labels.contains(&"security".to_string()) {
            details.push_str(
                " This task is security-sensitive and may require careful review before merging.",
            );
        }

        (code.to_string(), details)
    }
}

/// Parse feature_status.md and build an AcCoverageIndex.
///
/// The feature_status.md file is a markdown table with columns:
/// | AC ID | Story | Requirement | Status | Tests (executed/total) |
///
/// Status values: [PASS], [FAIL], [UNKNOWN]
///
/// Returns Ok with AcCoverageIndex if parsing succeeds, or an error otherwise.
/// Returns Ok with empty map if the file doesn't exist (not an error).
pub fn parse_feature_status(path: &Path) -> std::io::Result<AcCoverageIndex> {
    let mut index = AcCoverageIndex::new();

    if !path.exists() {
        return Ok(index);
    }

    let content = std::fs::read_to_string(path)?;

    for line in content.lines() {
        // Skip non-table lines
        if !line.starts_with("| AC-") {
            continue;
        }

        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() < 5 {
            continue; // Malformed line, skip it
        }

        let ac_id = parts[1];
        let story_id = parts[2];
        let requirement_id = parts[3];
        let status_str = parts[4];

        // Parse status: [PASS], [FAIL], or [UNKNOWN]
        let status = if status_str.contains("[PASS]") {
            // Extract executed/total from parts[5] if it exists
            let executed = 1;
            let total = 1;
            if parts.len() > 5 {
                if let Some(tests_part) = parts[5].split('/').next() {
                    if let Ok(ex) = tests_part.trim().parse::<u32>() {
                        AcExecutionStatus::Pass { executed: ex, total: 1 }
                    } else {
                        AcExecutionStatus::Pass { executed, total }
                    }
                } else {
                    AcExecutionStatus::Pass { executed, total }
                }
            } else {
                AcExecutionStatus::Pass { executed, total }
            }
        } else if status_str.contains("[FAIL]") {
            let executed = 1;
            let total = 1;
            if parts.len() > 5 {
                if let Some(tests_part) = parts[5].split('/').next() {
                    if let Ok(ex) = tests_part.trim().parse::<u32>() {
                        AcExecutionStatus::Fail { executed: ex, total: 1 }
                    } else {
                        AcExecutionStatus::Fail { executed, total }
                    }
                } else {
                    AcExecutionStatus::Fail { executed, total }
                }
            } else {
                AcExecutionStatus::Fail { executed, total }
            }
        } else if status_str.contains("[UNKNOWN]") {
            let total = if parts.len() > 5 {
                parts[5]
                    .split('/')
                    .next_back()
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .unwrap_or(0)
            } else {
                0
            };
            AcExecutionStatus::Unknown { total }
        } else {
            continue; // Unknown status, skip
        };

        index.insert(
            ac_id.to_string(),
            AcCoverage {
                ac_id: ac_id.to_string(),
                story_id: story_id.to_string(),
                requirement_id: requirement_id.to_string(),
                status,
            },
        );
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_priority_ordering() {
        assert!(HintPriority::High > HintPriority::Medium);
        assert!(HintPriority::Medium > HintPriority::Low);
    }

    #[test]
    fn test_ac_coverage_serde() {
        let coverage = AcCoverage {
            ac_id: "AC-TPL-001".to_string(),
            story_id: "US-TPL-001".to_string(),
            requirement_id: "REQ-TPL-HEALTH".to_string(),
            status: AcExecutionStatus::Pass { executed: 1, total: 1 },
        };

        let json = serde_json::to_string(&coverage).unwrap();
        assert!(json.contains("AC-TPL-001"));
    }

    #[test]
    fn test_hint_engine_basic() {
        let mut ac_index = AcCoverageIndex::new();
        ac_index.insert(
            "AC-TEST-001".to_string(),
            AcCoverage {
                ac_id: "AC-TEST-001".to_string(),
                story_id: "US-TEST-001".to_string(),
                requirement_id: "REQ-TEST-001".to_string(),
                status: AcExecutionStatus::Pass { executed: 1, total: 1 },
            },
        );

        let task = crate::Task {
            id: "TASK-TEST-001".to_string(),
            title: "Test task".to_string(),
            status: "Todo".to_string(),
            requirement: "REQ-TEST-001".to_string(),
            acs: vec!["AC-TEST-001".to_string()],
            labels: vec!["test".to_string()],
            owner: Some("alice".to_string()),
            docs: None,
            summary: "A test task".to_string(),
            recommended_flows: vec![],
            depends_on: vec![],
        };

        let engine = HintEngine::new(ac_index, vec![task]);
        let hints = engine.task_hints();

        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].title, "Test task");
        assert_eq!(hints[0].priority, HintPriority::Medium);
    }
}
