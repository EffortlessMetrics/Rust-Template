# spec-runtime

Runtime library for loading and operating on governance specifications.

## What It Is

`spec-runtime` is the core library for loading, validating, and querying the Rust-as-Spec platform's governance artifacts. It is designed to be:

- **Standalone**: Pure Rust with minimal dependencies
- **Authoritative**: Single source of truth for spec parsing and validation
- **Portable**: Used by CLI tools, HTTP services, and platform agents
- **Cacheable**: Designed for efficient loading at service startup

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `ledger` | Spec ledger parsing (stories, requirements, ACs) |
| `config` | Configuration schema validation via JSON Schema |
| `devex` | Developer experience flows and xtask command specs |
| `docs` | Documentation inventory and indexing |
| `graph` | Governance graph builder (stories → REQs → ACs → tests → docs) |
| `hints` | Agent hint engine for AC coverage and task prioritization |
| `tasks` | Work item tracking and sequencing |
| `schema` | Platform API schemas and endpoint metadata |
| `ui_contract` | UI surface contracts with stable identifiers |
| `service_metadata` | Service metadata loader |
| `k8s_iac` | Kubernetes IaC configuration support |
| `local_docker` | Local Docker configuration support |

### What It Is Not

- **No file I/O opinions**: Callers provide paths, this library reads and parses
- **No CLI**: Command-line interfaces live in `xtask`
- **No HTTP**: Web serving logic lives in `gov-http` and `app-http`
- **No AC coverage enforcement**: SLO thresholds and gates live in CI workflows

## Quick Start

### Loading All Specs

```rust
use spec_runtime::load_all_specs;
use std::path::Path;

let workspace_root = Path::new("/workspace");
let specs = load_all_specs(workspace_root)?;

println!("Loaded {} stories", specs.ledger.stories.len());
println!("Loaded {} flows", specs.devex.flows.len());
println!("Loaded {} docs", specs.docs.docs.len());
```

### Using RepoContext (Recommended)

The preferred pattern for kernel crates is to use `RepoContext` for path resolution:

```rust
use gov_model::RepoContext;
use spec_runtime::load_all_specs_with_context;

let ctx = RepoContext::new("/workspace");
let specs = load_all_specs_with_context(&ctx)?;
```

This ensures consistent path handling across all kernel crates.

### Loading Individual Specs

```rust
use spec_runtime::{load_spec_ledger, load_devex_flows, load_doc_index};

// Load spec ledger
let ledger = load_spec_ledger(Path::new("specs/spec_ledger.yaml"))?;
for story in &ledger.stories {
    println!("Story: {} - {}", story.id, story.title);
}

// Load devex flows
let flows = load_devex_flows(Path::new("specs/devex_flows.yaml"))?;
for flow in &flows.flows {
    println!("Flow: {} - {}", flow.id, flow.name);
}

// Load documentation index
let docs = load_doc_index(Path::new("specs/doc_index.yaml"))?;
for doc in &docs.docs {
    println!("Doc: {} - {}", doc.id, doc.file);
}
```

## Core Data Structures

### Spec Ledger

The governance ledger maps user stories → requirements → acceptance criteria → tests:

```rust
use spec_runtime::{SpecLedger, Story, Requirement, AcceptanceCriterion};

let ledger: SpecLedger = load_spec_ledger(path)?;

// Navigate the hierarchy
for story in &ledger.stories {
    println!("Story: {}", story.id);
    for req in &story.requirements {
        println!("  Requirement: {}", req.id);
        for ac in &req.acceptance_criteria {
            println!("    AC: {} - {}", ac.id, ac.text);
            for test in &ac.tests {
                println!("      Test: {} ({})", test.tag, test.test_type);
            }
        }
    }
}
```

### Configuration Validation

Validate runtime configuration against a JSON Schema:

```rust
use spec_runtime::{validate_config, ValidatedConfig};

let schema_path = Path::new("specs/config_schema.yaml");
let config_path = Path::new("config/local.yaml");

let config: ValidatedConfig = validate_config(schema_path, config_path)?;

println!("Environment: {:?}", config.env);
println!("HTTP Port: {}", config.http_port);
println!("Settings: {:?}", config.settings);
// Secrets are available but should be handled securely
```

**Schema Contract**: The config schema is the authoritative definition of what configuration is valid. Any config that passes validation is guaranteed to match the schema structure.

### Governance Graph

Build a queryable graph of governance relationships:

```rust
use spec_runtime::{build_graph, Graph};

let root = Path::new("/workspace");
let graph: Graph = build_graph(root)?;

// Query nodes and edges
for node in &graph.nodes {
    println!("Node: {} ({})", node.id, node.node_type);
}

for edge in &graph.edges {
    println!("Edge: {} -> {} ({})", edge.from, edge.to, edge.edge_type);
}
```

Graph types:
- **Nodes**: `Story`, `Requirement`, `AC`, `Test`, `Doc`, `Command`, `Flow`
- **Edges**: `has_requirement`, `has_ac`, `has_test`, `documented_by`, `implements`

### Agent Hints

The hint engine provides prioritized task recommendations:

```rust
use spec_runtime::{HintEngine, HintFilter, HintStatus};

let engine = HintEngine::new(root)?;

// Get all hints for tasks in Todo or InProgress status
let filter = HintFilter {
    status: Some(vec![HintStatus::Todo, HintStatus::InProgress]),
    ..Default::default()
};

let hints = engine.get_hints(&filter)?;

for hint in &hints {
    println!("Task: {} - {}", hint.target.task_id, hint.target.description);
    println!("  Priority: {:?}", hint.priority);
    println!("  Reason: {:?}", hint.reason);
    if let Some(links) = &hint.links {
        println!("  REQs: {:?}", links.requirement_ids);
        println!("  ACs: {:?}", links.ac_ids);
    }
}
```

**Hint priorities**:
- `High`: Task has failing ACs or is blocking other work
- `Medium`: Task has unknown AC coverage or is part of active flow
- `Low`: Task is ready to work but not urgent

### Tasks

Load and filter work items:

```rust
use spec_runtime::{load_tasks, TasksSpec};
use gov_model::TaskStatus;

let tasks_spec: TasksSpec = load_tasks(Path::new("specs/tasks.yaml"))?;

// Filter tasks by status
let todo_tasks: Vec<_> = tasks_spec.tasks.iter()
    .filter(|t| t.status == TaskStatus::Todo.to_string())
    .collect();

println!("Todo tasks: {}", todo_tasks.len());

// Find task dependencies
for task in &tasks_spec.tasks {
    if !task.depends_on.is_empty() {
        println!("Task {} depends on: {:?}", task.id, task.depends_on);
    }
}
```

## Index Structures

For fast lookups, use the index builders:

### AC ID Index

```rust
use spec_runtime::build_ac_id_index;

let ledger = load_spec_ledger(path)?;
let ac_index = build_ac_id_index(&ledger);

if let Some(ac_ref) = ac_index.get("AC-TPL-001") {
    println!("Found AC: {}", ac_ref.ac.text);
    println!("  Story: {}", ac_ref.story_id);
    println!("  Requirement: {}", ac_ref.req_id);
}
```

### Requirement ID Index

```rust
use spec_runtime::build_req_id_index;

let req_index = build_req_id_index(&ledger);

if let Some(req_ref) = req_index.get("REQ-TPL-CONFIG") {
    println!("Found requirement: {}", req_ref.req.title);
    println!("  Story: {}", req_ref.story_id);
    println!("  ACs: {}", req_ref.req.acceptance_criteria.len());
}
```

## Platform Schemas

Query available platform API schemas:

```rust
use spec_runtime::{get_all_schemas, get_schema_by_name};

// Get all schemas
let schemas = get_all_schemas();
println!("Available schemas: {}", schemas.schemas.len());

// Get specific schema
if let Some(schema) = get_schema_by_name("PlatformStatus") {
    println!("Schema: {}", schema.name);
    println!("Version: {}", schema.version);
    println!("Description: {}", schema.description);
    println!("Endpoint: {}", schema.endpoint);
}
```

**Schema versioning**: Each schema has a version field. Breaking changes require version bumps.

## RepoContext Integration

All loaders have `_with_context` variants for use with `RepoContext`:

```rust
use gov_model::RepoContext;
use spec_runtime::*;

let ctx = RepoContext::new("/workspace");

// Load individual specs
let ledger = load_spec_ledger_with_context(&ctx)?;
let flows = load_devex_flows_with_context(&ctx)?;
let docs = load_doc_index_with_context(&ctx)?;
let tasks = load_tasks_with_context(&ctx)?;
let metadata = load_service_metadata_with_context(&ctx)?;
let ui = load_ui_contract_with_context(&ctx)?;

// Validate config
let config = validate_config_with_context(
    &ctx,
    &ctx.workspace_root().join("config/local.yaml")
)?;
```

**Why RepoContext?** It provides:
- Unified workspace path resolution
- Consistent spec file location logic
- Better testability (mock paths easily)

## Consumers

This crate is used by:

| Crate | Usage |
|-------|-------|
| `xtask` | CLI commands (`ac-status`, `check`, `help-flows`) |
| `app-http` | Loading specs at service startup |
| `gov-http` | Platform endpoint data sources |
| `acceptance` | BDD test execution and coverage tracking |
| `governance` | Policy enforcement and validation |

## Stability

The spec file formats are versioned and considered public contracts:

| Spec File | Schema Version Field | Breaking Change Policy |
|-----------|---------------------|------------------------|
| `spec_ledger.yaml` | `metadata.schema_version` | Major version bump required |
| `devex_flows.yaml` | `schema_version` | Major version bump required |
| `doc_index.yaml` | `schema_version` | Major version bump required |
| `tasks.yaml` | `schema_version` | Major version bump required |
| `config_schema.yaml` | `$schema` | JSON Schema compatibility rules |

**Shape lock tests** enforce schema stability:
- `ledger::tests::spec_ledger_shape_is_stable`
- `devex::tests::devex_flows_shape_is_stable`
- `docs::tests::doc_index_shape_is_stable`

### AllSpecs Container

The `AllSpecs` struct aggregates the three core spec files:

```rust
pub struct AllSpecs {
    pub ledger: SpecLedger,
    pub devex: DevExFlows,
    pub docs: DocIndex,
}
```

This is the recommended way to load all governance data in one call.

## Default File Layout

`load_all_specs` expects this structure:

```
<workspace_root>/
├── specs/
│   ├── spec_ledger.yaml      # Governance ledger
│   ├── devex_flows.yaml      # Developer flows
│   ├── doc_index.yaml        # Documentation index
│   ├── tasks.yaml            # Work items
│   ├── config_schema.yaml    # Config schema
│   ├── service_metadata.yaml # Service metadata
│   └── ui_contract.yaml      # UI surface contract
└── config/
    └── local.yaml            # Runtime configuration
```

All paths are relative to the workspace root provided by the caller.

## Error Handling

All loaders return `anyhow::Result<T>` with context:

```rust
let ledger = load_spec_ledger(path)
    .context("Failed to load spec ledger")?;
```

Common errors:
- **File not found**: Spec file missing or path incorrect
- **Parse error**: YAML syntax error or schema violation
- **Validation error**: Config doesn't match schema

Use `.context()` to add clarity at call sites.

## See Also

- `docs/reference/spec-ledger-schema.md` - Spec ledger YAML schema
- `docs/reference/config-schema.md` - Configuration schema documentation
- `crates/gov-http/README.md` - HTTP endpoints using this library
- `crates/xtask/README.md` - CLI commands using this library
