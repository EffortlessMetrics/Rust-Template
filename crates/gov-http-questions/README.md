# gov-http-questions

Question endpoints for tracking design decisions and ambiguities.

## Purpose

This crate provides HTTP endpoints for managing question entries. Questions capture flow decision points, options, recommendations, and their resolutions during development.

## Endpoints

- `GET /questions` - List all questions (with optional status filter)
- `GET /questions/{id}` - Get a specific question by ID

## Dependencies

- `axum` - Web framework
- `http` - HTTP types
- `http-errors` - Error types and mapping (with axum feature)
- `platform-contract` - Platform contract types
- `gov-model` - Governance domain model
- `gov-http-core` - Shared router foundation
- `serde_yaml` - YAML parsing

## Usage

```rust
use gov_http_questions::router;
use axum::Router;
use gov_http_core::PlatformState;

// Compose questions router
let app = Router::new()
    .merge(router::<MyState>())
    .with_state(my_state);
```

## Data Types

### Question

Full question entry with all metadata:
- `id`: Unique identifier (e.g., "Q-001")
- `task_id`: Optional related task ID
- `req_ids`: Related requirement IDs
- `ac_ids`: Related AC IDs
- `refs`: Additional references
- `summary`: One-line summary
- `context`: Question context (flow, phase, etc.)
- `options`: Available options
- `recommendation`: Optional recommendation
- `created_by`: Who created the question
- `created_at`: Creation timestamp (ISO 8601)
- `status`: Status (defaults to "open")
- `resolution`: Optional resolution information

### QuestionContext

Context for the question:
- `flow`: Related development flow
- `phase`: Development phase
- `description`: Optional description
- `files_involved`: Files involved

### QuestionOption

An option for the question:
- `label`: Option label
- `description`: Option description
- `risk`: Optional risk assessment
- `reversible`: Whether the decision is reversible (defaults to true)

### Recommendation

Recommended option:
- `option_label`: Label of recommended option
- `rationale`: Rationale for recommendation
- `confidence`: Optional confidence level

### QuestionResolution

Resolution information:
- `resolved_by`: Who resolved it
- `resolved_at`: Resolution timestamp
- `chosen_option`: Which option was chosen
- `notes`: Optional notes

## Question File Format

Question entries are stored as YAML files in `questions/` directory:

```yaml
id: Q-EXAMPLE-001
task_id: implement_feature
req_ids:
  - REQ-001
ac_ids:
  - AC-001
summary: "Which database to use?"
context:
  flow: bundle
  phase: selection
  description: "Need to choose between PostgreSQL and MySQL"
  files_involved:
    - Cargo.toml
options:
  - label: "PostgreSQL"
    description: "Open source, mature"
    risk: "Learning curve"
    reversible: true
  - label: "MySQL"
    description: "Widely used"
    risk: "License cost"
    reversible: true
recommendation:
  option_label: "PostgreSQL"
  rationale: "Better fit for our use case"
  confidence: "high"
created_by: "dev-team"
created_at: "2025-11-26T00:00:00Z"
status: open
```
