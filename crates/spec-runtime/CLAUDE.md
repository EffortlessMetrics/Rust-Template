# spec-runtime – CLAUDE.md

**Tier:** Kernel (Layer 2)
**Publish:** Yes
**Dependencies:** gov-model, spec-types, serde, anyhow

## Purpose

Core orchestrator for loading and operating on governance specifications. Single source of truth for spec parsing, validation, and querying. Used by CLI, HTTP services, and agents.

## Key Exports

- `load_all_specs()` – Load all governance specs at once
- `load_spec_ledger()` – Parse spec_ledger.yaml
- `load_devex_flows()` – Parse devex_flows.yaml
- `load_doc_index()` – Parse doc_index.yaml
- `build_graph()` – Build governance graph
- `HintEngine` – Agent task recommendations
- `validate_config()` – Config schema validation
- Index builders: `build_ac_id_index()`, `build_req_id_index()`

## When to Modify

- Adding new spec file types
- Extending loader functionality
- Adding new graph node/edge types

## When NOT to Modify

- Adding CLI commands (those go in xtask)
- Adding HTTP routing (those go in gov-http)
- Adding AC coverage logic (that's in ac-kernel)

## Architectural Notes

- **Cacheable**: Designed for efficient startup loading
- **RepoContext integration**: All loaders have `_with_context` variants
- **Versioned schemas**: Spec files have `schema_version` fields
- **Shape-lock tests**: Schema stability enforced

## Key Files

- `specs/spec_ledger.yaml` – Governance ledger
- `specs/devex_flows.yaml` – Developer flows
- `specs/doc_index.yaml` – Documentation index
- `specs/tasks.yaml` – Work items

## Consumers

`xtask`, `app-http`, `gov-http`, `acceptance`

## See Also

- `README.md` in this crate for full API documentation
- `crates/ac-kernel/` for AC-specific logic
