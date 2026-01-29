# Glossary: Rust-as-Spec Domain Terms

This glossary defines key terms and concepts used in the Rust-as-Spec governance model and IDP Platform Cell.

## Core Concepts

### Acceptance Criteria (AC)

Concrete, testable requirements that define when a feature or requirement is considered complete. Every AC should ideally be mapped to at least one automated test (BDD, unit, or integration).
- **Format**: `AC-[CATEGORY]-[NUMBER]` (e.g., `AC-TPL-001`)
- **Storage**: `specs/spec_ledger.yaml`

### ADR (Architecture Decision Record)

A document that captures an important architectural decision made along with its context and consequences.
- **Storage**: `docs/adr/`
- **Command**: `cargo xtask adr-new "Title"`

### BDD (Behavior-Driven Development)

A software development process that encourages collaboration between developers, QA, and non-technical stakeholders. In this repo, BDD is implemented using Gherkin `.feature` files and the `cucumber` crate.
- **Storage**: `specs/features/*.feature`
- **Command**: `cargo xtask bdd`

### Context Bundle

A consolidated Markdown file containing relevant source code, specs, and documentation for a specific task. Used to provide "infinite context" to LLM agents.
- **Command**: `cargo xtask bundle <TASK_NAME>`

### IDP (Internal Developer Platform)

The set of tools, services, and processes that enable developer self-service. This repository is designed to be a "Platform Cell" that integrates with an IDP via standardized APIs and snapshots.

### Kernel

The foundational "governance engine" and template structure of the repository. It includes the `xtask` infrastructure, spec-loading logic, and baseline requirements.
- **Adoption**: See `docs/how-to/adopt-kernel.md`

### Platform Cell

A self-contained unit of the platform (like this service) that adheres to the platform's governance contracts and exposes standardized introspection APIs.

### Spec Ledger

The "single source of truth" for the repository's requirements. It contains stories, requirements, and acceptance criteria.
- **Storage**: `specs/spec_ledger.yaml`

### Selftest

A 12-step automated validation pipeline that verifies the repository's health, governance integrity, and compliance. It must pass before any PR is merged.
- **Command**: `cargo xtask selftest`

## Governance Artifacts

### Friction Log

A log used to track developer experience issues, tooling gaps, or process hurdles.
- **Command**: `cargo xtask friction-list`

### Question Artifact

A formal record of a design ambiguity or decision point encountered during development.
- **Command**: `cargo xtask questions-list`

### Task

A unit of work tracked in the repository, often linked to specific Requirements and ACs.
- **Storage**: `specs/tasks.yaml`
- **Command**: `cargo xtask tasks-list`

## Infrastructure

### Nix / Devshell

The tool used to provide a hermetic, reproducible development environment.
- **Command**: `nix develop`

### xtask

A Rust pattern for repository-specific automation tasks. In this repo, the `xtask` binary is the primary entry point for all developer workflows.
- **Command**: `cargo xtask --help`
