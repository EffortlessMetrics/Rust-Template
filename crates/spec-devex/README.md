# spec-devex

Developer experience flows and xtask command specifications.

## Purpose

This crate provides:
- DevEx flow types
- Command resolution
- Flow execution helpers

## Dependencies

- **spec-types**: Foundation types

## Design Principles

- **Minimal deps**: Only spec-types, serde, serde_yaml, thiserror, anyhow
- **Workflow layer**: Provides flow types and command resolution
- **No jsonschema**: Heavy dependencies are isolated to spec-schema

## Public API

### Types
- `DevExFlows`: Developer experience flows specification
- `CommandSpec`: Specification for an individual xtask command
- `DocsRequirement`: Documentation requirements for a command
- `FlowSpec`: Specification for a developer workflow
- `FlowExecution`: Flow execution result
- `FlowStep`: A step in flow execution
- `StepStatus`: Status of a flow step

### Functions
- `load_devex_flows(path)`: Load devex flows from YAML file
- `resolve_command(devex, command_id)`: Resolve a command by ID
- `get_commands_by_category(devex, category)`: Get all commands by category
- `get_required_commands(devex)`: Get all required commands
- `execute_flow(devex, flow_id)`: Execute a flow and return the result
- `get_required_flows(devex)`: Get all required flows

## Example

```rust
use spec_devex::{load_devex_flows, resolve_command};

let devex = load_devex_flows(Path::new("specs/devex_flows.yaml"))?;
let cmd = resolve_command(&devex, "check")?;

println!("Command: {} - {}", cmd.category, cmd.summary);
```
