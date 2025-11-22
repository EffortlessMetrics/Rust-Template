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
                    let test_label = if let Some(file) = &test_mapping.file {
                        format!("{} - {}", test_mapping.tag, file)
                    } else {
                        test_mapping.tag.clone()
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

#[derive(Debug)]
pub struct InvariantViolation {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

pub fn check_invariants(
    graph: &Graph,
    devex: &DevExFlows,
    ledger: &SpecLedger,
) -> Result<(), Vec<InvariantViolation>> {
    let mut violations = Vec::new();

    // 1. REQ must_have_ac -> at least one AC
    for req in graph.nodes.iter().filter(|n| n.node_type == "requirement") {
        if !req.meta.must_have_ac {
            continue;
        }

        let has_ac = graph.edges.iter().any(|e| e.source == req.id && e.edge_type == "contains");

        if !has_ac {
            violations.push(InvariantViolation {
                code: "REQ_HAS_NO_AC".into(),
                message: format!("Requirement {} has no ACs in graph", req.id),
            });
        }
    }

    // 2. AC has tests - validate that ACs with test mappings have non-empty tests array
    for story in &ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                if ac.tests.is_empty() {
                    violations.push(InvariantViolation {
                        code: "AC_HAS_NO_TEST".into(),
                        message: format!("Acceptance criterion {} has no test mappings", ac.id),
                    });
                }
            }
        }
    }

    // 3. DevEx commands reachable
    for cmd_name in devex.commands.keys() {
        let cmd_id = format!("cmd:{}", cmd_name);
        let is_reachable = graph.edges.iter().any(|e| e.target == cmd_id);
        if !is_reachable {
            if let Some(cmd_spec) = devex.commands.get(cmd_name) {
                if cmd_spec.required {
                    violations.push(InvariantViolation {
                        code: "COMMAND_UNREACHABLE".into(),
                        message: format!(
                            "Required command '{}' is not used in any flow or task",
                            cmd_name
                        ),
                    });
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

impl Graph {
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
                template_version: "3.2.0".to_string(),
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
                            },
                            TestMapping {
                                test_type: "integration".to_string(),
                                tag: "@test-tag-2".to_string(),
                                file: None,
                            },
                        ],
                    }],
                }],
            }],
        };

        let devex = DevExFlows {
            schema_version: "1.0".to_string(),
            template_version: "3.2.0".to_string(),
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        let docs = crate::DocIndex {
            schema_version: "1.0".to_string(),
            template_version: "3.2.0".to_string(),
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
            template_version: "3.2.0".to_string(),
            commands: std::collections::HashMap::new(),
            flows: std::collections::HashMap::new(),
        };

        // Create an empty graph
        let graph = Graph { nodes: Vec::new(), edges: Vec::new() };

        // Test case 1: AC with tests should pass
        let ledger_with_tests = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.2.0".to_string(),
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
                        }],
                    }],
                }],
            }],
        };

        let result = check_invariants(&graph, &devex, &ledger_with_tests);
        assert!(result.is_ok(), "AC with tests should pass validation");

        // Test case 2: AC without tests should fail
        let ledger_without_tests = SpecLedger {
            metadata: Metadata {
                schema_version: "1.0".to_string(),
                template_version: "3.2.0".to_string(),
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

        let result = check_invariants(&graph, &devex, &ledger_without_tests);
        assert!(result.is_err(), "AC without tests should fail validation");

        if let Err(violations) = result {
            assert_eq!(violations.len(), 1);
            assert_eq!(violations[0].code, "AC_HAS_NO_TEST");
            assert!(violations[0].message.contains("AC-TEST-002"));
        }
    }
}
