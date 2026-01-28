# spec-graph

Build and query the dependency graph from spec data.

## Purpose

This crate provides graph building and query functions for the governance
traceability graph connecting:
- Stories → Requirements → Acceptance Criteria → Tests → Docs
- Flows → Commands

## Dependencies

- **spec-types**: Foundation types
- **spec-ledger**: Ledger data structures

## Design Principles

- **No jsonschema**: Heavy dependencies are isolated to spec-schema
- **No axum**: HTTP/web dependencies are isolated to higher-level crates
- **Foundation**: Depends on spec-ledger, not on higher-level crates

## Public API

### Types
- `Graph`: Governance graph with nodes and edges
- `Node`: Graph node (story, requirement, AC, test, document, command, flow)
- `Edge`: Graph edge connecting two nodes
- `NodeMeta`: Node metadata for validation
- `InvariantViolation`: Violation from invariant checking
- `InvariantStatus`: Status of an invariant check
- `InvariantReport`: Report from invariant checking

### Functions
- `build_graph(ledger, devex, docs)`: Build governance graph
- `check_invariants(graph, devex, ledger)`: Check graph invariants

### Graph Methods
- `Graph::to_mermaid()`: Export graph as Mermaid diagram

## Example

```rust
use spec_graph::{build_graph, check_invariants};

let graph = build_graph(&ledger, &devex, &docs)?;
let report = check_invariants(&graph, &devex, &ledger);

if !report.passed {
    eprintln!("Violations: {:?}", report.violations);
}

// Export as Mermaid
println!("{}", graph.to_mermaid());
```
