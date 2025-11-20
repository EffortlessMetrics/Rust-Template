use anyhow::Result;
use serde::Serialize;

use crate::{DevExFlows, DocIndex, SpecLedger};

#[derive(Debug, Serialize, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

pub fn build_graph(ledger: &SpecLedger, devex: &DevExFlows, docs: &DocIndex) -> Result<Graph> {
    let mut graph = Graph { nodes: Vec::new(), edges: Vec::new() };

    // Add stories, requirements, and ACs
    for story in &ledger.stories {
        graph.nodes.push(Node {
            id: story.id.clone(),
            label: story.title.clone(),
            node_type: "story".to_string(),
            url: None,
        });

        for req in &story.requirements {
            graph.nodes.push(Node {
                id: req.id.clone(),
                label: req.title.clone(),
                node_type: "requirement".to_string(),
                url: None,
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
                });

                graph.edges.push(Edge {
                    source: req.id.clone(),
                    target: ac.id.clone(),
                    edge_type: "contains".to_string(),
                });
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

pub fn check_invariants(graph: &Graph, devex: &DevExFlows) -> Result<()> {
    let mut errors = Vec::new();

    // 1. Every requirement with certain tags must have at least one AC
    let req_nodes: Vec<&Node> =
        graph.nodes.iter().filter(|n| n.node_type == "requirement").collect();
    for req in req_nodes {
        // In a real implementation we'd check tags, but for now we assume all requirements in the graph matter
        // Check if it has an outgoing edge to an AC
        let has_ac = graph.edges.iter().any(|e| e.source == req.id && e.edge_type == "contains");
        if !has_ac {
            // Check if it's a requirement that SHOULD have ACs (e.g. platform/structural)
            // For this simplified check, we'll just warn or error if it's a core requirement
            if req.id.starts_with("REQ-TPL") {
                errors.push(format!("Requirement {} has no ACs", req.id));
            }
        }
    }

    // 2. Every AC must have at least one test edge (or be manually verified)
    // This is harder to check purely from the graph as we don't explicitly model test nodes yet in build_graph
    // But we can check if ACs are leaf nodes (which they shouldn't be if they have tests)
    // For now, we'll skip this strictly or implement a basic check if we added test nodes.
    // Since we didn't add test nodes in build_graph yet, let's skip this one for now to avoid false positives.

    // 3. Every command in devex_flows must be reachable
    // We check if command nodes are orphans (no incoming edges from flows/tasks)
    // Actually, commands are usually leaf nodes in our current graph (Flow -> Command).
    // So we check if they have INCOMING edges.
    for cmd_name in devex.commands.keys() {
        let cmd_id = format!("cmd:{}", cmd_name);
        let is_reachable = graph.edges.iter().any(|e| e.target == cmd_id);
        if !is_reachable {
            // Check if it's required
            if let Some(cmd_spec) = devex.commands.get(cmd_name) {
                if cmd_spec.required {
                    errors.push(format!(
                        "Required command '{}' is not used in any flow or task",
                        cmd_name
                    ));
                }
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Graph invariants failed:\n- {}", errors.join("\n- "));
    }

    Ok(())
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
