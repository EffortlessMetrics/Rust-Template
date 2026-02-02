use anyhow::Result;
use serde::Serialize;

use crate::{DevExFlows, DocIndex, SpecLedger};

#[derive(Debug, Serialize, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct NodeMeta {
    #[serde(default)]
    pub must_have_ac: bool,
}

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

#[derive(Debug, Serialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

// ... (Edge struct remains same)

#[tracing::instrument(skip(ledger, devex, docs))]
pub fn build_graph(ledger: &SpecLedger, devex: &DevExFlows, docs: &DocIndex) -> Result<Graph> {
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

#[derive(Debug, Serialize, Clone)]
pub struct InvariantStatus {
    pub code: String,
    pub description: String,
    pub passed: bool,
    pub checked_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct InvariantReport {
    pub checked_at: String,
    pub invariants: Vec<InvariantStatus>,
    pub violations: Vec<InvariantViolation>,
    pub passed: bool,
}

#[tracing::instrument(skip(graph, devex, ledger))]
pub fn check_invariants(graph: &Graph, devex: &DevExFlows, ledger: &SpecLedger) -> InvariantReport {
    let mut violations = Vec::new();
    let mut invariants = Vec::new();

    // Optimize lookups by pre-indexing edges
    // - req_sources_with_ac: Set of source IDs that have a "contains" edge (used for REQ check)
    // - reachable_targets: Set of target IDs that are reachable (used for CMD check)
    let mut req_sources_with_ac = std::collections::HashSet::new();
    let mut reachable_targets = std::collections::HashSet::new();

    for edge in &graph.edges {
        if edge.edge_type == "contains" {
            req_sources_with_ac.insert(edge.source.as_str());
        }
        reachable_targets.insert(edge.target.as_str());
    }

    // 1. REQ must_have_ac -> at least one AC
    let mut req_ac_checked = 0;
    let mut req_ac_violations = Vec::new();

    for req in graph.nodes.iter().filter(|n| n.node_type == "requirement") {
        if !req.meta.must_have_ac {
            continue;
        }
        req_ac_checked += 1;

        let has_ac = req_sources_with_ac.contains(req.id.as_str());

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

    // 2. AC has tests - validate that ACs with test mappings have non-empty tests array
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

    for cmd_name in devex.commands.keys() {
        if let Some(cmd_spec) = devex.commands.get(cmd_name)
            && cmd_spec.required
        {
            cmd_reachable_checked += 1;
            let cmd_id = format!("cmd:{}", cmd_name);
            let is_reachable = reachable_targets.contains(cmd_id.as_str());

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

impl Graph {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{Metadata, TestMapping};
    use crate::{AcceptanceCriterion, DevExFlows, Requirement, SpecLedger, Story};

    /// AC-TPL-GRAPH-AC-HAS-TEST: Validates that every AC with a tests entry
    /// in the ledger produces a graph node with at least one edge to a test node.
    #[test]
    fn ac_with_tests_produces_graph_node_and_edge() {
        // 1. Build a tiny SpecLedger in-memory with a single AC that has tests
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
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let docs = crate::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            docs: vec![],
        };

        // 2. Run build_graph(...)
        let graph = build_graph(&ledger, &devex, &docs).expect("build_graph should succeed");

        // 3. Assert there is a node with id == "AC-XYZ"
        let ac_node = graph.nodes.iter().find(|n| n.id == "AC-XYZ");
        assert!(ac_node.is_some(), "Graph should contain a node for AC-XYZ");
        assert_eq!(ac_node.unwrap().node_type, "ac");

        // 4. Assert at least one edge from that node to a test node
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

    #[test]
    fn graph_invariants_ac_has_test() {
        // Create a minimal DevExFlows with no commands
        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        // Create an empty graph
        let graph = Graph { nodes: Vec::new(), edges: Vec::new() };

        // Test case 1: AC with tests should pass
        let ledger_with_tests = SpecLedger {
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
                        text: "Test AC with tests".to_string(),
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

        let report = check_invariants(&graph, &devex, &ledger_with_tests);
        assert!(report.passed, "AC with tests should pass validation");

        // Test case 2: AC without tests should fail
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

    /// AC-TPL-GRAPH-REQ-HAS-AC: Validates that requirements with must_have_ac: true
    /// have at least one AC in the graph.
    #[test]
    fn graph_invariants_req_has_ac() {
        use std::collections::HashMap;

        // Create a minimal DevExFlows
        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: HashMap::new(),
            flows: HashMap::new(),
        };

        let docs = crate::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            docs: vec![],
        };

        // Test case 1: Requirement with must_have_ac and an AC should pass
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

        // Test case 2: Requirement with must_have_ac but no AC should fail
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

    /// AC-TPL-GRAPH-COMMAND-REACHABLE: Validates that required DevEx commands
    /// are reachable (used in at least one flow).
    #[test]
    fn graph_invariants_command_reachable() {
        use crate::devex::{CommandSpec, FlowSpec};
        use std::collections::HashMap;

        let docs = crate::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            docs: vec![],
        };

        let ledger = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.3.1".to_string(),
                last_updated: "2025-11-22".to_string(),
                description: "Test ledger".to_string(),
            },
            stories: vec![],
        };

        // Test case 1: Required command that is used in a flow should pass
        let mut commands_valid = HashMap::new();
        commands_valid.insert(
            "check".to_string(),
            CommandSpec {
                category: "validation".to_string(),
                summary: "Run checks".to_string(),
                required: true,
                docs: Default::default(),
            },
        );

        let mut flows_valid = HashMap::new();
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

        let devex_valid = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: commands_valid,
            flows: flows_valid,
        };

        let graph = build_graph(&ledger, &devex_valid, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex_valid, &ledger);
        assert!(report.passed, "Required command used in flow should pass validation");

        // Test case 2: Required command not used in any flow should fail
        let mut commands_invalid = HashMap::new();
        commands_invalid.insert(
            "orphan-cmd".to_string(),
            CommandSpec {
                category: "validation".to_string(),
                summary: "Orphaned command".to_string(),
                required: true,
                docs: Default::default(),
            },
        );

        let devex_invalid = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: commands_invalid,
            flows: HashMap::new(),
        };

        let graph =
            build_graph(&ledger, &devex_invalid, &docs).expect("build_graph should succeed");
        let report = check_invariants(&graph, &devex_invalid, &ledger);
        assert!(!report.passed, "Required command not in any flow should fail validation");
        assert!(report.violations.iter().any(|v| v.code == "COMMAND_UNREACHABLE"));
        let cmd_violation =
            report.violations.iter().find(|v| v.code == "COMMAND_UNREACHABLE").unwrap();
        assert!(cmd_violation.message.contains("orphan-cmd"));
    }

    /// AC-TPL-GRAPH-SELFTEST: Validates that the mermaid export produces valid
    /// Mermaid graph syntax.
    #[test]
    fn graph_export_mermaid() {
        use std::collections::HashMap;

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
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            commands: HashMap::new(),
            flows: HashMap::new(),
        };

        let docs = crate::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "3.3.1".to_string(),
            docs: vec![],
        };

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
