//! Governance graph builder and query functions.
//!
//! This crate builds and queries the dependency graph from spec data.
//! It connects stories → requirements → acceptance criteria → tests → docs.
//!
//! # Design Principles
//!
//! - **No jsonschema**: Heavy dependencies are isolated to spec-schema
//! - **No axum**: HTTP/web dependencies are isolated to higher-level crates
//! - **Foundation**: Depends on spec-ledger and spec-types
//!
//! # Example
//!
//! ```ignore
//! use spec_graph::{build_graph, check_invariants};
//!
//! let graph = build_graph(&ledger, &devex, &docs)?;
//! let report = check_invariants(&graph, &devex, &ledger);
//! ```

#![allow(missing_docs)]

use serde::Serialize;
use spec_ledger::SpecLedger;
use spec_types::SpecResult;

// ============================================================================
// Public Types
// ============================================================================

/// Governance graph with nodes and edges.
#[derive(Debug, Serialize, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// Node metadata for validation.
#[derive(Debug, Serialize, Clone, Default)]
pub struct NodeMeta {
    #[serde(default)]
    pub must_have_ac: bool,
}

/// A graph node (story, requirement, AC, test, document, command, flow).
#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub meta: NodeMeta,
}

/// A graph edge connecting two nodes.
#[derive(Debug, Serialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

/// Invariant violation from graph validation.
#[derive(Debug, Serialize, Clone)]
pub struct InvariantViolation {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// Status of an invariant check.
#[derive(Debug, Serialize, Clone)]
pub struct InvariantStatus {
    pub code: String,
    pub description: String,
    pub passed: bool,
    pub checked_count: usize,
}

/// Report from invariant checking.
#[derive(Debug, Serialize, Clone)]
pub struct InvariantReport {
    pub checked_at: String,
    pub invariants: Vec<InvariantStatus>,
    pub violations: Vec<InvariantViolation>,
    pub passed: bool,
}

// ============================================================================
// DevEx Types (minimal for graph building)
// ============================================================================

/// Command specification (minimal version for graph building).
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub category: String,
    pub summary: String,
    pub required: bool,
}

/// Flow specification (minimal version for graph building).
#[derive(Debug, Clone)]
pub struct FlowSpec {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub documented_in: Vec<String>,
    pub steps: Vec<String>,
}

/// DevEx flows (minimal version for graph building).
#[derive(Debug, Clone)]
pub struct DevExFlows {
    pub commands: std::collections::HashMap<String, CommandSpec>,
    pub flows: std::collections::HashMap<String, FlowSpec>,
}

// ============================================================================
// Doc Types (minimal for graph building)
// ============================================================================

/// Documentation entry (minimal version for graph building).
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub id: String,
    pub file: String,
    pub doc_type: String,
    pub requirements: Vec<String>,
}

/// Doc index (minimal version for graph building).
#[derive(Debug, Clone)]
pub struct DocIndex {
    pub docs: Vec<DocEntry>,
}

// ============================================================================
// Graph Building
// ============================================================================

/// Build the governance graph from ledger, devex, and docs.
///
/// Creates nodes for:
/// - Stories, requirements, acceptance criteria
/// - Tests (from AC test mappings)
/// - Documents (from doc index)
/// - Commands and flows (from devex)
///
/// Creates edges for:
/// - Story → requirement (contains)
/// - Requirement → AC (contains)
/// - AC → test (tested_by)
/// - Document → requirement (documents)
/// - Flow → command (executes)
#[tracing::instrument(skip(ledger, devex, docs))]
pub fn build_graph(ledger: &SpecLedger, devex: &DevExFlows, docs: &DocIndex) -> SpecResult<Graph> {
    let mut graph = Graph { nodes: Vec::new(), edges: Vec::new() };

    // Add stories, requirements, and ACs
    for story in &ledger.stories {
        graph.nodes.push(Node {
            id: story.id.clone(),
            label: story.title.clone(),
            node_type: "story".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });

        for req in &story.requirements {
            graph.nodes.push(Node {
                id: req.id.clone(),
                label: req.title.clone(),
                node_type: "requirement".to_string(),
                url: None,
                meta: NodeMeta { must_have_ac: req.must_have_ac },
            });

            graph.edges.push(Edge {
                source: story.id.clone(),
                target: req.id.clone(),
                edge_type: "contains".to_string(),
            });

            for ac in &req.acceptance_criteria {
                graph.nodes.push(Node {
                    id: ac.id.clone(),
                    label: ac.text.clone(),
                    node_type: "ac".to_string(),
                    url: None,
                    meta: NodeMeta::default(),
                });

                graph.edges.push(Edge {
                    source: req.id.clone(),
                    target: ac.id.clone(),
                    edge_type: "contains".to_string(),
                });

                // Add test nodes and edges for each test mapping
                for (idx, test_mapping) in ac.tests.iter().enumerate() {
                    let test_node_id = format!("{}:test:{}", ac.id, idx);
                    let base_label = test_mapping
                        .module
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| test_mapping.tag.clone());
                    let test_label = if let Some(file) = &test_mapping.file {
                        format!("{base_label} - {file}")
                    } else {
                        base_label
                    };

                    graph.nodes.push(Node {
                        id: test_node_id.clone(),
                        label: test_label,
                        node_type: "test".to_string(),
                        url: test_mapping.file.as_ref().map(|f| format!("file://{}", f)),
                        meta: NodeMeta::default(),
                    });

                    graph.edges.push(Edge {
                        source: ac.id.clone(),
                        target: test_node_id,
                        edge_type: "tested_by".to_string(),
                    });
                }
            }
        }
    }

    // Add documents
    for doc in &docs.docs {
        graph.nodes.push(Node {
            id: doc.file.clone(),
            label: doc.id.clone(),
            node_type: "document".to_string(),
            url: Some(format!("file://{}", doc.file)),
            meta: NodeMeta::default(),
        });

        for req_id in &doc.requirements {
            graph.edges.push(Edge {
                source: doc.file.clone(),
                target: req_id.clone(),
                edge_type: "documents".to_string(),
            });
        }
    }

    // Add commands
    for cmd_name in devex.commands.keys() {
        graph.nodes.push(Node {
            id: format!("cmd:{}", cmd_name),
            label: format!("cargo xtask {}", cmd_name),
            node_type: "command".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });
    }

    // Add flows
    for (flow_id, flow) in &devex.flows {
        let flow_node_id = format!("flow:{}", flow_id);
        graph.nodes.push(Node {
            id: flow_node_id.clone(),
            label: flow.name.clone(),
            node_type: "flow".to_string(),
            url: None,
            meta: NodeMeta::default(),
        });

        for step in &flow.steps {
            graph.edges.push(Edge {
                source: flow_node_id.clone(),
                target: format!("cmd:{}", step),
                edge_type: "executes".to_string(),
            });
        }
    }

    Ok(graph)
}

// ============================================================================
// Invariant Checking
// ============================================================================

/// Check graph invariants.
///
/// Validates:
/// - Requirements with must_have_ac=true have at least one AC
/// - ACs have test mappings
/// - Required commands are used in at least one flow
#[tracing::instrument(skip(graph, devex, ledger))]
pub fn check_invariants(graph: &Graph, devex: &DevExFlows, ledger: &SpecLedger) -> InvariantReport {
    let mut violations = Vec::new();
    let mut invariants = Vec::new();

    // 1. REQ must_have_ac -> at least one AC
    let mut req_ac_checked = 0;
    let mut req_ac_violations = Vec::new();

    for req in graph.nodes.iter().filter(|n| n.node_type == "requirement") {
        if !req.meta.must_have_ac {
            continue;
        }
        req_ac_checked += 1;

        let has_ac = graph.edges.iter().any(|e| e.source == req.id && e.edge_type == "contains");

        if !has_ac {
            req_ac_violations.push(InvariantViolation {
                code: "REQ_HAS_NO_AC".into(),
                message: format!("Requirement {} has no ACs in graph", req.id),
            });
        }
    }

    invariants.push(InvariantStatus {
        code: "REQ_HAS_AC".into(),
        description: "Requirements with must_have_ac=true have at least one AC".into(),
        passed: req_ac_violations.is_empty(),
        checked_count: req_ac_checked,
    });
    violations.extend(req_ac_violations);

    // 2. AC has tests
    let mut ac_test_checked = 0;
    let mut ac_test_violations = Vec::new();

    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                ac_test_checked += 1;
                if ac.tests.is_empty() {
                    ac_test_violations.push(InvariantViolation {
                        code: "AC_HAS_NO_TEST".into(),
                        message: format!("Acceptance criterion {} has no test mappings", ac.id),
                    });
                }
            }
        }
    }

    invariants.push(InvariantStatus {
        code: "AC_HAS_TEST".into(),
        description: "Acceptance criteria have at least one test mapping".into(),
        passed: ac_test_violations.is_empty(),
        checked_count: ac_test_checked,
    });
    violations.extend(ac_test_violations);

    // 3. DevEx commands reachable
    let mut cmd_reachable_checked = 0;
    let mut cmd_reachable_violations = Vec::new();

    for (cmd_name, cmd_spec) in &devex.commands {
        if cmd_spec.required {
            cmd_reachable_checked += 1;
            let cmd_id = format!("cmd:{}", cmd_name);
            let is_reachable = graph.edges.iter().any(|e| e.target == cmd_id);

            if !is_reachable {
                cmd_reachable_violations.push(InvariantViolation {
                    code: "COMMAND_UNREACHABLE".into(),
                    message: format!(
                        "Required command '{}' is not used in any flow or task",
                        cmd_name
                    ),
                });
            }
        }
    }

    invariants.push(InvariantStatus {
        code: "COMMAND_REACHABLE".into(),
        description: "Required commands are used in at least one flow".into(),
        passed: cmd_reachable_violations.is_empty(),
        checked_count: cmd_reachable_checked,
    });
    violations.extend(cmd_reachable_violations);

    let passed = violations.is_empty();
    let checked_at = chrono::Utc::now().to_rfc3339();

    InvariantReport { checked_at, invariants, violations, passed }
}

// ============================================================================
// Graph Export
// ============================================================================

impl Graph {
    /// Export graph as Mermaid diagram.
    #[tracing::instrument(skip(self), fields(node_count = self.nodes.len(), edge_count = self.edges.len()))]
    pub fn to_mermaid(&self) -> String {
        let mut out = String::new();
        out.push_str("graph TD\n");

        // Emit node declarations
        for node in &self.nodes {
            let id_safe = mermaid_id(&node.id);
            let label = mermaid_label(&node.id, &node.label);
            out.push_str(&format!("  {id_safe}[\"{label}\"]\n"));
        }

        out.push('\n');

        // Emit edges
        for edge in &self.edges {
            let src = mermaid_id(&edge.source);
            let tgt = mermaid_id(&edge.target);
            let rel = &edge.edge_type;
            out.push_str(&format!("  {src} -->|\"{rel}\"| {tgt}\n"));
        }

        out
    }
}

fn mermaid_id(id: &str) -> String {
    // Mermaid node IDs must be simple identifiers: replace non-alnum with '_'
    id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
}

fn mermaid_label(id: &str, label: &str) -> String {
    // Use "ID\nTitle" style; escape quotes and newlines
    let mut clean_label = label.replace('"', "\\\"");
    // Truncate overly long labels for readability
    if clean_label.len() > 80 {
        clean_label.truncate(77);
        clean_label.push_str("...");
    }
    format!("{id}\\n{clean_label}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use spec_ledger::{AcceptanceCriterion, Metadata, Requirement, SpecLedger, Story, TestMapping};

    /// AC with tests produces graph node and edge.
    #[test]
    fn ac_with_tests_produces_graph_node_and_edge() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger for graph AC-has-test validation".to_string(),
            },
            stories: vec![Story {
                id: "US-TEST-001".to_string(),
                title: "Test Story".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-TEST-001".to_string(),
                    title: "Test Requirement".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![AcceptanceCriterion {
                        id: "AC-XYZ".to_string(),
                        text: "Test AC with tests mapping".to_string(),
                        tests: vec![
                            TestMapping {
                                test_type: "unit".to_string(),
                                tag: "test_tag_1".to_string(),
                                file: Some("tests/test_file.rs".to_string()),
                                module: None,
                            },
                            TestMapping {
                                test_type: "integration".to_string(),
                                tag: "@test-tag-2".to_string(),
                                file: None,
                                module: None,
                            },
                        ],
                    }],
                }],
            }],
        };

        let devex = DevExFlows {
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let docs = DocIndex { docs: vec![] };

        let graph = build_graph(&ledger, &devex, &docs).expect("build_graph should succeed");

        // Assert there is a node with id == "AC-XYZ"
        let ac_node = graph.nodes.iter().find(|n| n.id == "AC-XYZ");
        assert!(ac_node.is_some(), "Graph should contain a node for AC-XYZ");
        assert_eq!(ac_node.unwrap().node_type, "ac");

        // Assert at least one edge from that node to a test node
        let edges_from_ac: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.source == "AC-XYZ" && e.edge_type == "tested_by")
            .collect();

        assert!(
            !edges_from_ac.is_empty(),
            "AC-XYZ should have at least one 'tested_by' edge to a test node"
        );

        // Verify we have exactly 2 test edges (one for each test mapping)
        assert_eq!(edges_from_ac.len(), 2, "AC-XYZ should have exactly 2 test edges");

        // Verify the test nodes exist
        for edge in &edges_from_ac {
            let test_node = graph.nodes.iter().find(|n| n.id == edge.target);
            assert!(test_node.is_some(), "Test node {} should exist in graph", edge.target);
            assert_eq!(test_node.unwrap().node_type, "test");
        }
    }

    /// Graph invariants: AC has test.
    #[test]
    fn graph_invariants_ac_has_test() {
        let devex = DevExFlows {
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let graph = Graph { nodes: Vec::new(), edges: Vec::new() };

        // Test case: AC without tests should fail
        let ledger_without_tests = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![Story {
                id: "US-TEST-002".to_string(),
                title: "Test Story 2".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-TEST-002".to_string(),
                    title: "Test Requirement 2".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![AcceptanceCriterion {
                        id: "AC-TEST-002".to_string(),
                        text: "Test AC without tests".to_string(),
                        tests: vec![],
                    }],
                }],
            }],
        };

        let report = check_invariants(&graph, &devex, &ledger_without_tests);
        assert!(!report.passed, "AC without tests should fail validation");
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].code, "AC_HAS_NO_TEST");
        assert!(report.violations[0].message.contains("AC-TEST-002"));
    }

    /// Graph invariants: REQ has AC.
    #[test]
    fn graph_invariants_req_has_ac() {
        let devex = DevExFlows {
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let docs = DocIndex { docs: vec![] };

        // Test case: Requirement with must_have_ac and an AC should pass
        let ledger_valid = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![Story {
                id: "US-TEST-001".to_string(),
                title: "Test Story".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-TEST-001".to_string(),
                    title: "Test Requirement".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![AcceptanceCriterion {
                        id: "AC-TEST-001".to_string(),
                        text: "Test AC".to_string(),
                        tests: vec![TestMapping {
                            test_type: "unit".to_string(),
                            tag: "test_tag".to_string(),
                            file: None,
                            module: None,
                        }],
                    }],
                }],
            }],
        };

        let graph = build_graph(&ledger_valid, &devex, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex, &ledger_valid);
        assert!(report.passed, "Requirement with AC should pass validation");

        // Test case: Requirement with must_have_ac but no AC should fail
        let ledger_invalid = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![Story {
                id: "US-TEST-002".to_string(),
                title: "Test Story 2".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-TEST-002".to_string(),
                    title: "Test Requirement 2".to_string(),
                    tags: vec![],
                    must_have_ac: true,
                    acceptance_criteria: vec![],
                }],
            }],
        };

        let graph =
            build_graph(&ledger_invalid, &devex, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex, &ledger_invalid);
        assert!(!report.passed, "Requirement without AC should fail validation");
        assert!(report.violations.iter().any(|v| v.code == "REQ_HAS_NO_AC"));
        let req_violation = report.violations.iter().find(|v| v.code == "REQ_HAS_NO_AC").unwrap();
        assert!(req_violation.message.contains("REQ-TEST-002"));
    }

    /// Graph invariants: command reachable.
    #[test]
    fn graph_invariants_command_reachable() {
        let docs = DocIndex { docs: vec![] };

        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![],
        };

        // Test case: Required command that is used in a flow should pass
        let mut commands_valid = std::collections::HashMap::new();
        commands_valid.insert(
            "check".to_string(),
            CommandSpec {
                category: "validation".to_string(),
                summary: "Run checks".to_string(),
                required: true,
            },
        );

        let mut flows_valid = std::collections::HashMap::new();
        flows_valid.insert(
            "validate".to_string(),
            FlowSpec {
                name: "Validate".to_string(),
                description: "Run validation".to_string(),
                required: false,
                documented_in: vec![],
                steps: vec!["check".to_string()],
            },
        );

        let devex_valid = DevExFlows { commands: commands_valid, flows: flows_valid };

        let graph = build_graph(&ledger, &devex_valid, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex_valid, &ledger);
        assert!(report.passed, "Required command used in flow should pass validation");

        // Test case: Required command not used in any flow should fail
        let mut commands_invalid = std::collections::HashMap::new();
        commands_invalid.insert(
            "orphan-cmd".to_string(),
            CommandSpec {
                category: "validation".to_string(),
                summary: "Orphaned command".to_string(),
                required: true,
            },
        );

        let devex_invalid =
            DevExFlows { commands: commands_invalid, flows: std::collections::HashMap::new() };

        let graph =
            build_graph(&ledger, &devex_invalid, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex_invalid, &ledger);
        assert!(!report.passed, "Required command not in any flow should fail validation");
        assert!(report.violations.iter().any(|v| v.code == "COMMAND_UNREACHABLE"));
        let cmd_violation =
            report.violations.iter().find(|v| v.code == "COMMAND_UNREACHABLE").unwrap();
        assert!(cmd_violation.message.contains("orphan-cmd"));
    }

    /// Mermaid export produces valid syntax.
    #[test]
    fn graph_export_mermaid() {
        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![Story {
                id: "US-TEST-001".to_string(),
                title: "Test Story".to_string(),
                requirements: vec![Requirement {
                    id: "REQ-TEST-001".to_string(),
                    title: "Test Requirement".to_string(),
                    tags: vec![],
                    must_have_ac: false,
                    acceptance_criteria: vec![AcceptanceCriterion {
                        id: "AC-TEST-001".to_string(),
                        text: "Test AC".to_string(),
                        tests: vec![TestMapping {
                            test_type: "unit".to_string(),
                            tag: "test_tag".to_string(),
                            file: None,
                            module: None,
                        }],
                    }],
                }],
            }],
        };

        let devex = DevExFlows {
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let docs = DocIndex { docs: vec![] };

        let graph = build_graph(&ledger, &devex, &docs).expect("build_graph should succeed");
        let mermaid = graph.to_mermaid();

        // Validate basic Mermaid structure
        assert!(mermaid.starts_with("graph TD\n"), "Mermaid should start with 'graph TD'");
        assert!(mermaid.contains("US_TEST_001"), "Mermaid should contain story node");
        assert!(mermaid.contains("REQ_TEST_001"), "Mermaid should contain requirement node");
        assert!(mermaid.contains("AC_TEST_001"), "Mermaid should contain AC node");
        assert!(mermaid.contains("-->"), "Mermaid should contain edges");
    }
}
