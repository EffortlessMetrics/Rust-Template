# LLM Context Bundle

**Git SHA:** 225db0b2500e8d926c0f295843017e5b08f4f28c

**Description:** Context for implementing a single acceptance criterion end-to-end.

**Max bytes:** 250000

---

# FILE: specs/spec_ledger.yaml

```
# Spec Ledger: Story → Requirement → Acceptance Criteria → Tests mapping
# This file defines the canonical traceability from user stories to test coverage

metadata:
  schema_version: "1.0"
  template_version: "3.3.3"
  last_updated: "2025-11-26"
  description: "Template core capabilities - v2.4.0 includes health, version, errors, metrics, and supply chain hardening"
  # Template-wide ADRs (not tied to specific stories/requirements)
  adrs:
    - ADR-0002  # Nix-first dev environment (applies to entire template)
    - ADR-0004  # Policy and LLM governance (applies to entire template)
    - ADR-0006  # Supply chain hardening (applies to entire template)

stories:
  # Template Core - Keep and extend these in your service
  - id: US-TPL-001
    title: "Service Core Capabilities"
    adr: ADR-0001  # Hexagonal architecture
    requirements:
      - id: REQ-TPL-HEALTH
        title: "Health Check Endpoint"
        tags: [platform, structural]
        must_have_ac: true
        adr: ADR-0003  # Spec and BDD as source of truth
        acceptance_criteria:
          - id: AC-TPL-001
            text: "GET /health returns 200 with status 'ok' when service is healthy"
            tags: [kernel]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-TPL-005", file: "specs/features/template_core.feature" } ]
            adr: ADR-0005  # Selftest as single gate
      - id: REQ-TPL-VERSION
        title: "Version Information Endpoint"
        tags: [platform]
        must_have_ac: true
        adr: ADR-0003  # Spec and BDD as source of truth
        acceptance_criteria:
          - id: AC-TPL-002
            text: "GET /version returns build information including version and git SHA"
            tags: [kernel]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-TPL-002", file: "specs/features/template_core.feature" } ]
            adr: ADR-0005  # Selftest as single gate
      - id: REQ-TPL-ERROR-HANDLING
        title: "Error Response Envelope"
        tags: [platform, structural]
        must_have_ac: true
        adr: ADR-0003  # Spec and BDD as source of truth
        acceptance_criteria:
          - id: AC-TPL-003
            text: "All 4xx/5xx responses include an error code, message, and request ID"
            tags: [kernel]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-TPL-003", file: "specs/features/template_core.feature" } ]
            adr: ADR-0005  # Selftest as single gate
          - id: AC-TPL-004
            text: "Handlers propagate or generate X-Request-ID and expose it in responses"
            tags: [kernel]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-TPL-004", file: "specs/features/template_core.feature" } ]
            adr: ADR-0005  # Selftest as single gate
      - id: REQ-TPL-METRICS
        title: "Prometheus Metrics Endpoint"
        tags: [platform]
        must_have_ac: true
        adr: ADR-0003  # Spec and BDD as source of truth
        acceptance_criteria:
          - id: AC-TPL-007
            text: "GET /metrics returns Prometheus-formatted metrics including http_requests_total"
            tags: [kernel]
            must_have_ac: true
            tests: [ { type: bdd, tag: "@AC-TPL-007", file: "specs/features/metrics.feature" } ]
            adr: ADR-0005  # Selftest as single gate

  - id: US-TPL-PLT-001
    title: "Platform: Developer Experience & Governance"
    description: >
      The template MUST provide discoverable, documented, and tested developer
      workflows (onboarding, AC-first development, security auditing, releases)
      as first-class platform features, with flows defined as spec and enforced by CI.
    adr: 
      - ADR-0002  # Nix-first
      - ADR-0005  # Single selftest gate
      - ADR-0006  # Supply chain
      - ADR-0007  # Dependency health
    requirements:

      - id: REQ-PLT-ONBOARDING
        title: "New developer onboarding is guided and validated"
        tags: [platform, devex]
        must_have_ac: true
        description: >
          New developers on any machine can validate their environment,
          discover workflows, and reach first green check within 15 minutes.
        adr: [ADR-0002, ADR-0017]
        ci_workflows:
          - CI-NIX
          - CI-TIER1-SELFTEST
        acceptance_criteria:
          - id: AC-PLT-001
            text: "`cargo xtask doctor` validates Rust, Nix, conftest, git and provides next-steps guidance"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-001", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_check_rust_version_accepts_valid_versions", module: "commands::doctor::tests", file: "crates/xtask/src/commands/doctor.rs" }
              - { type: unit, tag: "test_doctor_command_exists", module: "commands::doctor::tests", file: "crates/xtask/src/commands/doctor.rs" }
          
          - id: AC-PLT-002
            text: "`cargo xtask help-flows` renders categorized command map from specs/devex_flows.yaml"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-002", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_help_flows_command_exists", module: "commands::help_flows::tests", file: "crates/xtask/src/commands/help_flows.rs" }
              - { type: unit, tag: "test_devex_spec_contains_required_categories", module: "commands::help_flows::tests", file: "crates/xtask/src/commands/help_flows.rs" }

          - id: AC-PLT-003
            text: "`cargo xtask check` runs fmt + clippy + tests as fast dev loop"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-003", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_check_command_exists", module: "commands::check::tests", file: "crates/xtask/src/commands/check.rs" }
              - { type: unit, tag: "test_check_options_low_resource_mode", module: "commands::check::tests", file: "crates/xtask/src/commands/check.rs" }

          - id: AC-PLT-018
            text: "`cargo xtask dev-up` runs doctor + install-hooks + check and displays next steps"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-018", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-021
            text: >
              `cargo xtask service-init` updates service_metadata.yaml,
              README, and CLAUDE.md with a new service ID, name, and
              description, and `/platform/status` reflects the new identity.
            adr: ADR-0022
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-021", file: "specs/features/xtask_devex.feature" }

      - id: REQ-PLT-DESIGN-SCAFFOLDING
        title: "Design decisions and ACs are easy to create correctly"
        tags: [platform, devex]
        must_have_ac: true
        description: >
          Scaffolding commands make the right path (ADR-first, AC-first) the easy path,
          reducing error rates and cognitive load.
        adr: ADR-0001  # Design history
        acceptance_criteria:
          - id: AC-PLT-004
            text: "`cargo xtask adr-new <title>` creates numbered ADR from template with metadata"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-004", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_adr_new_command_exists", module: "commands::adr_new::tests", file: "crates/xtask/src/commands/adr_new.rs" }
              - { type: unit, tag: "test_slug_generation_from_title", module: "commands::adr_new::tests", file: "crates/xtask/src/commands/adr_new.rs" }
              - { type: unit, tag: "test_adr_id_formatting", module: "commands::adr_new::tests", file: "crates/xtask/src/commands/adr_new.rs" }

          - id: AC-PLT-005
            text: "`cargo xtask ac-new <ID> <desc>` rejects duplicate IDs and generates YAML snippet"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-005", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_ac_new_command_exists", module: "commands::ac_new::tests", file: "crates/xtask/src/commands/ac_new.rs" }
              - { type: unit, tag: "test_ac_id_validation_requires_prefix", module: "commands::ac_new::tests", file: "crates/xtask/src/commands/ac_new.rs" }
              - { type: unit, tag: "test_ac_entry_generation", module: "commands::ac_new::tests", file: "crates/xtask/src/commands/ac_new.rs" }

      - id: REQ-PLT-AC-GOVERNANCE
        title: "AC classification is governed and versioned"
        tags: [platform, governance]
        must_have_ac: false
        description: >
          Changes to acceptance criterion classification (must_have_ac field) are treated
          as kernel contract changes. The rules, process, and rationale are documented
          and enforced.
        adr: ADR-0005
        docs:
          - docs/feature_status_notes.md
          - docs/how-to/change-acceptance-criterion.md
        acceptance_criteria:
          - id: AC-PLT-AC-DEMOTION-GOVERNED
            text: >
              Changes to must_have_ac for any acceptance criterion are treated as kernel
              contract changes and are documented with an ADR and kernel version update.
            tags: [template, governance]
            must_have_ac: false
            tests:
              - { type: ci, tag: "ac_demotion_governed", note: "Governance policy" }

      - id: REQ-PLT-SECURITY-GOVERNANCE
        title: "Security and dependency health are first-class gates"
        tags: [platform, security]
        must_have_ac: true
        description: >
          Security auditing is not an afterthought but a required step in
          dependency changes and releases, with clear recovery guidance.
        adr: ADR-0007
        ci_workflows:
          - CI-SECURITY
          - CI-POLICY-VERIFY
          - CI-MAINTENANCE-PIN-ACTIONS
        acceptance_criteria:
          - id: AC-PLT-006
            text: "`cargo xtask audit` runs cargo-audit + cargo-deny with repo policy (deny.toml)"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-006", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_audit_command_exists", module: "commands::audit::tests", file: "crates/xtask/src/commands/audit.rs" }
              - { type: unit, tag: "test_audit_checks_for_tools", module: "commands::audit::tests", file: "crates/xtask/src/commands/audit.rs" }

          - id: AC-PLT-007
            text: "`cargo xtask audit` provides 4-step recovery guidance on failure"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-007", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-008
            text: "`cargo xtask sbom-local` generates SPDX JSON to target/sbom.spdx.json"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-008", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "test_sbom_local_command_exists", module: "commands::sbom_local::tests", file: "crates/xtask/src/commands/sbom_local.rs" }
              - { type: unit, tag: "test_sbom_json_structure", module: "commands::sbom_local::tests", file: "crates/xtask/src/commands/sbom_local.rs" }
              - { type: unit, tag: "test_package_id_replacement_logic", module: "commands::sbom_local::tests", file: "crates/xtask/src/commands/sbom_local.rs" }

      - id: REQ-PLT-DOCS-CONSISTENCY
        title: "Documentation alignment is automatically validated"
        tags: [platform, docs]
        must_have_ac: true
        description: >
          Version drift between spec_ledger, README, CLAUDE is caught by CI,
          not discovered months later.
        adr: ADR-0005
        ci_workflows:
          - CI-DOCS
        acceptance_criteria:
          - id: AC-PLT-009
            text: "`cargo xtask docs-check` validates version alignment across spec_ledger, README, CLAUDE"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-009", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-010
            text: "`cargo xtask docs-check` regenerates feature_status and fails on dirty git tree"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-010", file: "specs/features/xtask_devex.feature" }

      - id: REQ-PLT-RELEASE-SAFETY
        title: "Releases are guided, validated, and never skip steps"
        tags: [platform, release]
        must_have_ac: true
        description: >
          Release process is a single command sequence with automated validation,
          making "forgot to run audit" or "dirty tree" impossible.
        adr: ADR-0005
        ci_workflows:
          - CI-SUPPLY-CHAIN
          - CI-RELEASE-SBOM-SIGN
        acceptance_criteria:
          - id: AC-PLT-011
            text: "`cargo xtask release-prepare X.Y.Z` updates spec_ledger, README, CLAUDE, CHANGELOG"
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Release tooling is advanced workflow."
            tests:
              - { type: integration, tag: "@AC-PLT-011", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-012
            text: "`cargo xtask release-verify` runs selftest + audit + docs-check + clean tree"
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Release tooling is advanced workflow."
            tests:
              - { type: integration, tag: "@AC-PLT-012", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-013
            text: "`cargo xtask release-verify` provides git command sequence on success"
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Release tooling is advanced workflow."
            tests:
              - { type: integration, tag: "@AC-PLT-013", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-REL-BUNDLE
        title: "Release evidence bundle generation"
        tags: [platform, release, devex]
        must_have_ac: true
        description: >
          The system MUST be able to produce a structured evidence bundle for a given
          version, suitable for generating release notes.
        adr: ADR-0005
        docs: docs/design/release-bundling.md
        acceptance_criteria:
          - id: AC-TPL-REL-EVIDENCE
            text: "`cargo xtask release-bundle X.Y.Z` writes `release_evidence/vX.Y.Z.md` containing: all tasks completed in this version, linked REQs/ACs/ADRs, git log since last tag, selftest summary, policy status, resolved friction entries."
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Release tooling is advanced workflow."
            tests:
              - { type: integration, tag: "release_bundle_generation", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-REL-CHANGELOG
            text: "Evidence file includes distinct sections (Tasks, Specs/ACs, ADRs, Git log, Governance signals) adequate for LLM formatting into Keep a Changelog format."
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Release tooling is advanced workflow."
            tests:
              - { type: integration, tag: "release_bundle_structure", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-KERNEL-CONTRACT-EMITTED
            text: "`cargo xtask release-bundle X.Y.Z` writes `release_evidence/kernel_contract.vX.Y.Z.json` describing xtask commands, /platform/* endpoints, and governance schemas for that version."
            tags: [template, release]
            must_have_ac: false
            note: "Template feature - kernel contract emission for versioned platform interfaces."
            tests:
              - { type: integration, tag: "@AC-TPL-KERNEL-CONTRACT-EMITTED", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-EXAMPLE-FORK
        title: "Template fork examples"
        tags: [template, docs, example]
        must_have_ac: true
        description: >
          The template SHOULD provide working examples of fork customization
          to demonstrate extensibility and best practices.
        acceptance_criteria:
          - id: AC-TPL-EXAMPLE-FORK-BUILDS
            text: "`examples/fork-customization/` builds and passes its own selftest in CI, demonstrating a working fork."
            tags: [template, example]
            must_have_ac: false
            note: "CI-only: tests cross-workspace behavior. Example fork must build independently in CI."
            tests:
              - { type: ci, tag: "example_fork_ci" }

      - id: REQ-PLT-DEVEX-CONTRACT
        title: "DevEx flows and commands are spec-backed and CI-enforced"
        tags: [platform, devex]
        must_have_ac: true
        description: >
          The template defines its own workflows in machine-readable spec,
          tests that required commands exist, and fails CI if docs/code drift from spec.
        adr: [ADR-0005, ADR-0017]
        ci_workflows:
          - CI-TIER1-SELFTEST
          - CI-TEMPLATE-SELFTEST
          - CI-GOVERNANCE
          - CI-GHERKIN
          - CI-AC
          - CI-COVERAGE
          - CI-FEATURES
          - CI-LINTS
          - CI-MSRV
          - CI-FLAGS
          - CI-FLAGS-WARN
          - CI-DOCS
          - CI-SCOPE-GUARD
        acceptance_criteria:
          - id: AC-PLT-014
            text: "Canonical flows and commands are defined in specs/devex_flows.yaml"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "devex_flows_schema_valid", module: "devex::tests::devex_flows_schema_valid", file: "crates/spec-runtime/src/devex.rs" }

          - id: AC-PLT-015
            text: "`cargo xtask selftest` enforces devex contract (required commands exist)"
            tags: [kernel]
            must_have_ac: true
            note: "Unit tests verify contract enforcement logic; BDD @ci-only to avoid recursive selftest."
            adr: ADR-0017
            tests:
              - { type: unit, tag: "devex_contract_enforced_missing_commands", module: "commands::selftest::tests::devex_contract_enforced_missing_commands", file: "crates/xtask/src/commands/selftest.rs" }
              - { type: unit, tag: "devex_contract_enforced_all_present", module: "commands::selftest::tests::devex_contract_enforced_all_present", file: "crates/xtask/src/commands/selftest.rs" }
              - { type: unit, tag: "devex_contract_real_spec_satisfied", module: "commands::selftest::tests::devex_contract_real_spec_satisfied", file: "crates/xtask/src/commands/selftest.rs" }
              - { type: integration, tag: "@AC-PLT-015", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-016
            text: "`cargo xtask ci-local` orchestrates doctor + selftest + audit + docs-check"
            tags: [template, orchestration]
            must_have_ac: false
            note: "Template feature - not required for day-0 kernel. Advanced orchestration workflow."
            adr: ADR-0017
            tests:
              - { type: integration, tag: "@AC-PLT-016", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-019
            text: "`cargo xtask selftest` displays a condensed summary with clear pass/fail indicators for all 10 steps"
            tags: [kernel]
            must_have_ac: true
            note: "Unit tests verify summary structure and status tracking; BDD @ci-only to avoid recursive selftest."
            adr: ADR-0017
            tests:
              - { type: unit, tag: "selftest_summary_has_ten_steps", module: "commands::selftest::tests::selftest_summary_has_ten_steps", file: "crates/xtask/src/commands/selftest.rs" }
              - { type: unit, tag: "selftest_results_track_status", module: "commands::selftest::tests::selftest_results_track_status", file: "crates/xtask/src/commands/selftest.rs" }
              - { type: integration, tag: "@AC-PLT-019", file: "specs/features/xtask_devex.feature" }

          - id: AC-PLT-020
            text: "`XTASK_LOW_RESOURCES=1` environment variable skips resource-intensive steps in selftest for CI/constrained environments"
            tags: [kernel]
            must_have_ac: true
            adr: ADR-0017
            tests:
              - { type: integration, tag: "@AC-PLT-020", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-CLI-JSON-OUTPUT
            text: >
              For core reporting commands (`ac-status`, `version`,
              `friction-list`, `questions-list`, `fork-list`), passing
              `--json` produces a single valid JSON document on stdout
              with a stable top-level shape, and exit codes follow the
              success/failure of the operation.
            tags: [kernel, devex, ai, idp]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-TPL-CLI-JSON-OUTPUT", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-AUTOMATION-BEHAVIOUR
        title: "Automation-safe xtask behaviour"
        tags: [platform, devex, ci, ai]
        must_have_ac: true
        description: >
          Core xtask commands used by CI and agents MUST be non-interactive
          in automation mode and return stable exit codes that reflect
          success or failure.
        acceptance_criteria:
          - id: AC-TPL-XTASK-NONINTERACTIVE
            text: >
              For commands covered by the DevEx contract (doctor, check,
              selftest, ac-status, ac-coverage, bundle, version,
              friction-*, questions-*, fork-*), setting CI=1 or
              XTASK_NONINTERACTIVE=1 guarantees:
              - no interactive prompts, and
              - exit code 0 on success, non-zero on failure.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-XTASK-NONINTERACTIVE", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-BDD-HARNESS
        title: "Deterministic BDD harness semantics"
        tags: [devex, testing]
        must_have_ac: true
        description: >
          The BDD acceptance test harness MUST have deterministic exit code
          behavior that distinguishes between test failures and skipped tests,
          enabling reliable CI/CD integration and agent workflows.
        acceptance_criteria:
          - id: AC-TPL-BDD-EXIT-CODES
            text: >
              The acceptance test binary returns exit 0 when all non-@wip
              scenarios pass (regardless of skipped scenarios), and returns
              non-zero only if at least one non-@wip scenario fails.
            tags: [template, testing, harness]
            must_have_ac: false
            note: "Meta-harness contract: verified by CI and selftest [BDD-PASS] output. Cannot test harness with harness."
            tests:
              # The harness behavior is documented in the acceptance crate:
              # crates/acceptance/tests/acceptance.rs::bdd_exit_code_respects_wip
              # This is intentionally NOT mapped as a unit test since ac-status
              # excludes the acceptance crate to avoid test harness recursion.
              - { type: ci, tag: "bdd_exit_code_ci", note: "CI validates [BDD-PASS] output" }

      - id: REQ-PLT-STATUS-CLI
        title: "CLI governance status dashboard"
        tags: [platform, devex, observability]
        must_have_ac: true
        description: >
          Developers and agents need a quick way to check governance health,
          task status, and get oriented without navigating the web UI or
          multiple API endpoints.
        adr: ADR-0005
        docs: docs/design/status-cli.md
        acceptance_criteria:
          - id: AC-PLT-017
            text: "`cargo xtask status` displays version, REQ/AC/task counts, selftest status, and suggested next tasks"
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-PLT-017", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-PLATFORM-INTROSPECTION
        title: "Platform Introspection API"
        tags: [platform, devex, api]
        must_have_ac: true
        description: >
          The template MUST provide HTTP endpoints to introspect its own
          governance state (graph, tasks, flows, docs), enabling agents
          and dashboards to understand the platform without parsing files.
        adr: [ADR-0001]
        acceptance_criteria:
          - id: AC-TPL-PLATFORM-GRAPH
            text: "GET /platform/graph returns the full governance graph in JSON format."
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-GRAPH", file: "specs/features/platform_introspection.feature" }
          - id: AC-TPL-PLATFORM-DEVEX
            text: "GET /platform/devex/flows returns the canonical flows definition."
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-DEVEX", file: "specs/features/platform_introspection.feature" }
          - id: AC-TPL-PLATFORM-DOCS
            text: "GET /platform/docs/index returns the documentation index."
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-DOCS", file: "specs/features/platform_introspection.feature" }
          - id: AC-TPL-POLICY-STATUS-OVERVIEW
            text: >
              GET /platform/status includes governance.policies.status field
              derived from the last policy-test run (pass/fail/unknown),
              read from target/policy_status.json.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-STATUS" }

      - id: REQ-TPL-PLATFORM-SCHEMA
        title: "Machine-readable platform contract"
        tags: [platform, api, structural]
        must_have_ac: true
        description: >
          Platform APIs MUST expose a machine-readable schema so IDP dashboards,
          agents, and platform tooling can consume endpoints without scraping examples.
        adr: [ADR-0001]
        ci_workflows:
          - CI-OPENAPI
        acceptance_criteria:
          - id: AC-TPL-PLATFORM-SCHEMA
            text: >
              GET /platform/schema (or /platform/openapi) returns a JSON
              schema/OpenAPI document that includes /platform/status,
              /platform/graph, /platform/tasks, and /platform/agent/hints.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-SCHEMA", file: "specs/features/platform_schema.feature" }

      - id: REQ-TPL-METADATA-CONSISTENT
        title: "Platform metadata is coherent and exposed"
        tags: [platform, docs, metadata]
        must_have_ac: true
        description: >
          Service metadata MUST be complete in source files and surfaced
          consistently via platform status and the UI so fleet tooling can rely on it.
        acceptance_criteria:
          - id: AC-TPL-METADATA-COMPLETE
            text: >
              service_metadata.yaml includes service_id, template_version,
              URLs, and tags; /platform/status returns the same identifiers;
              the UI links to runbook, roadmap, agent guide, feature status,
              and platform support docs.
            adr: ADR-0022
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-METADATA-COMPLETE", file: "specs/features/platform_schema.feature" }

      - id: REQ-TPL-PLATFORM-AUTH
        title: "Platform introspection supports authenticated mode"
        tags: [platform, security]
        must_have_ac: true
        description: >
          Platform endpoints SHOULD support authenticated production mode while
          remaining open for local/dev use.
        acceptance_criteria:
          - id: AC-TPL-PLATFORM-AUTH-BASIC
            text: >
              When PLATFORM_AUTH_MODE=basic, write endpoints under /platform/*
              reject unauthenticated requests with 401/403 and accept requests
              with the configured credential header; read endpoints may remain
              open or use the same guard.
            tags: [future, security]
            must_have_ac: false
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-AUTH-BASIC", file: "specs/features/platform_security.feature" }

      - id: REQ-TPL-LOG-HYGIENE
        title: "Secrets are not exposed in status or logs"
        tags: [platform, security, observability]
        must_have_ac: true
        description: >
          Platform status, UI output, and selftest summaries MUST avoid leaking
          secret values or PII; only redacted markers or key names may appear.
        ci_workflows:
          - CI-PRIVACY
        acceptance_criteria:
          - id: AC-TPL-LOG-NO-SECRETS
            text: >
              Rendering of /platform/status and UI dashboards redacts or omits
              secret values, and selftest/log output does not include raw secret
              data from config.
            tags: [future, security]
            must_have_ac: false
            tests:
              - { type: unit, tag: "log_hygiene_redacts_secrets", module: "platform::tests::log_hygiene_redacts_secrets" }
              - { type: bdd, tag: "@AC-TPL-LOG-NO-SECRETS", file: "specs/features/platform_security.feature" }

      - id: REQ-TPL-SUGGEST-NEXT
        title: "Task-Aware Next-Step Suggestions"
        tags: [platform, devex, structural]
        must_have_ac: true
        description: >
          The template MUST provide a canonical way for humans and agents
          to get a suggested next-step sequence for a given task, derived
          from specs/tasks.yaml and specs/devex_flows.yaml.
        adr: [ADR-0001]
        docs:
          - design_doc: "docs/design/suggest-next.md"
        acceptance_criteria:
          - id: AC-TPL-SUGGEST-NEXT-CLI
            text: >
              cargo xtask suggest-next --task <ID> prints a structured
              sequence of recommended commands and edits for the given
              task, based on its recommended_flows.
            tags: [future, recommended]
            must_have_ac: false
            note: "Optional CLI convenience; not kernel - CLAUDE.md mentions alongside bundles, not in day-0 flows"
            tests:
              - { type: bdd, tag: "@AC-TPL-SUGGEST-NEXT-CLI", file: "specs/features/platform_rounding.feature" }

          - id: AC-TPL-SUGGEST-NEXT-HTTP
            text: >
              GET /platform/tasks/suggest-next?task=<ID> returns a JSON
              payload with task metadata and a recommended_sequence of
              steps (commands/edits) for that task.
            tags: [future]
            must_have_ac: false
            tests:
              - { type: bdd, tag: "@AC-TPL-SUGGEST-NEXT-HTTP", file: "specs/features/platform_rounding.feature" }

      - id: REQ-TPL-GOV-ARTIFACTS
        title: "Governance artifacts are structured and surfaced"
        tags: [platform, governance, ai, devex]
        must_have_ac: true
        description: >
          Questions, friction entries, and fork metadata MUST be stored as
          structured artifacts under version control, accessible via CLI
          and HTTP, and included in governance views.
        acceptance_criteria:
          - id: AC-TPL-QUESTIONS-LOGGED
            text: >
              Ambiguity during automated flows or suggest-next emits a
              structured question (file/PR comment/status entry) that can be
              surfaced to humans or agents without halting progress.
            tags: [template, governance]
            must_have_ac: false
            note: "Template feature - question artifacts. Verified via BDD in actual workspace."
            tests:
              - { type: bdd, tag: "@AC-TPL-QUESTIONS-LOGGED", file: "specs/features/questions.feature" }

          - id: AC-TPL-GOV-FRICTION
            text: >
              Friction log entries are stored as structured files under
              friction/, can be created and listed via `cargo xtask
              friction-new`/`friction-list`, and are exposed via
              /platform/friction and /platform/friction/{id}.
            tags: [template, governance]
            must_have_ac: false
            note: "Template feature - friction log. Verified via BDD in actual workspace."
            tests:
              - { type: bdd, tag: "@AC-TPL-GOV-FRICTION",
                  file: "specs/features/friction.feature" }

          - id: AC-TPL-GOV-FORKS
            text: >
              Fork metadata is stored under forks/fork_registry.yaml,
              can be managed via `cargo xtask fork-register`/`fork-list`,
              and is exposed via /platform/forks and /platform/forks/{name}.
            tags: [template, governance]
            must_have_ac: false
            note: "Template feature - fork registry. Verified via BDD in actual workspace."
            tests:
              - { type: bdd, tag: "@AC-TPL-GOV-FORKS",
                  file: "specs/features/forks.feature" }

          - id: AC-TPL-ARTIFACTS-HAVE-REFS
            text: >
              Questions and friction entries support a 'refs' field for REQ-*/AC-* IDs,
              allowing governance artifacts to reference the contracts they relate to.
            tags: [kernel, traceability, governance]
            must_have_ac: true
            note: "Kernel contract verified by unit + light BDD. Forks may demote via must_have_ac: false if not using refs."
            tests:
              - { type: bdd, tag: "@AC-TPL-ARTIFACTS-HAVE-REFS", file: "specs/features/questions.feature" }
              - { type: unit, tag: "artifacts_have_refs",
                  module: "xtask::commands::questions::tests::artifacts_have_refs",
                  file: "crates/xtask/src/commands/questions.rs" }

      - id: REQ-TPL-FORK-VISIBILITY
        title: "Forks are visible in governance"
        tags: [governance, idp]
        must_have_ac: true
        description: >
          Fork registry MUST be visible in platform governance views so
          agents and IDPs can discover known forks without reading raw YAML.
        acceptance_criteria:
          - id: AC-TPL-FORKS-STATUS-SUMMARY
            text: >
              /platform/status includes governance.forks.total and a forks.ids
              array when forks/fork_registry.yaml exists, and
              `cargo xtask fork-list --json` reflects that state.
            tags: [kernel, governance, idp]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-FORKS-STATUS-SUMMARY",
                  file: "specs/features/platform_schema.feature" }

      - id: REQ-TPL-FLOW-IDEMPOTENCY
        title: "Platform flows are safe to rerun"
        tags: [platform, devex, ai]
        must_have_ac: true
        description: >
          Self-healing and agent-driven flows should be idempotent so running
          them multiple times does not duplicate artifacts or corrupt state.
        acceptance_criteria:
          - id: AC-TPL-FLOW-IDEMPOTENT
            text: >
              Running cargo xtask selftest or cargo xtask suggest-next multiple
              times without changes produces stable outputs and no duplicate
              artifacts.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-FLOW-IDEMPOTENT", file: "specs/features/flow_idempotency.feature" }

      - id: REQ-TPL-GRAPH-INVARIANTS
        title: "Governance Graph Invariants"
        tags: [platform, structural]
        must_have_ac: true
        description: >
          The governance graph MUST satisfy baseline structural invariants,
          so that missing coverage and dead configuration are detected
          automatically.
        adr: [ADR-0001]
        docs: ["DESIGN-TPL-GRAPH-INVARIANTS-001"]
        acceptance_criteria:
          - id: AC-TPL-GRAPH-REQ-HAS-AC
            text: >
              Every requirement with tags including platform, structural,
              security, devex, docs, or release has at least one AC node
              in the graph.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "graph_invariants_req_has_ac", module: "graph::tests::graph_invariants_req_has_ac", file: "crates/spec-runtime/src/graph.rs" }

          - id: AC-TPL-GRAPH-AC-HAS-TEST
            text: >
              Every AC with a tests mapping in spec_ledger.yaml has at
              least one test node linked in the graph.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "ac_with_tests_produces_graph_node_and_edge", module: "graph::tests::ac_with_tests_produces_graph_node_and_edge", file: "crates/spec-runtime/src/graph.rs" }

          - id: AC-TPL-GRAPH-COMMAND-REACHABLE
            text: >
              Every command declared in specs/devex_flows.yaml is either
              referenced by a flow or explicitly marked internal; no
              orphan commands exist.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "graph_invariants_command_reachable", module: "graph::tests::graph_invariants_command_reachable", file: "crates/spec-runtime/src/graph.rs" }

          - id: AC-TPL-GRAPH-SELFTEST
            text: >
              cargo xtask selftest validates graph invariants and outputs
              'Graph invariants satisfied' when all checks pass.
            tags: [kernel]
            must_have_ac: true
            note: "Graph invariants validated by unit tests in spec-runtime. BDD @ci-only for full selftest integration."
            tests:
              # Unit tests for each invariant that selftest validates
              - { type: unit, tag: "graph_invariants_req_has_ac", module: "graph::tests::graph_invariants_req_has_ac", file: "crates/spec-runtime/src/graph.rs" }
              - { type: unit, tag: "ac_with_tests_produces_graph_node_and_edge", module: "graph::tests::ac_with_tests_produces_graph_node_and_edge", file: "crates/spec-runtime/src/graph.rs" }
              - { type: unit, tag: "graph_invariants_command_reachable", module: "graph::tests::graph_invariants_command_reachable", file: "crates/spec-runtime/src/graph.rs" }
              # BDD for full selftest integration (CI-only)
              - { type: bdd, tag: "@AC-TPL-GRAPH-SELFTEST", file: "specs/features/graph_invariants.feature" }

      - id: REQ-TPL-PLATFORM-UI
        title: "Platform Web UI"
        tags: [platform, devex, ui]
        must_have_ac: true
        description: >
          The template MUST provide a web-based UI for visualizing and
          interacting with the governance platform, served by the same
          binary that provides the API endpoints.
        adr: [ADR-0004]
        docs: [DESIGN-TPL-PLATFORM-UI-001]
        acceptance_criteria:
          - id: AC-TPL-PLATFORM-UI-DASHBOARD
            text: >
              GET / or /ui serves an HTML dashboard showing platform status,
              including governance health metrics from /platform/status.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-UI-DASHBOARD", file: "specs/features/platform_ui.feature" }

          - id: AC-TPL-PLATFORM-UI-GRAPH
            text: >
              The UI provides a graph visualization rendering the governance
              graph (stories, requirements, ACs, docs, commands) using Mermaid.js.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-UI-GRAPH", file: "specs/features/platform_ui.feature" }

          - id: AC-TPL-PLATFORM-UI-FLOWS
            text: >
              The UI provides a flows and tasks view displaying DevEx flows
              and available tasks from the platform APIs.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-UI-FLOWS", file: "specs/features/platform_ui.feature" }

      - id: REQ-TPL-PLATFORM-TASKS
        title: "Platform Tasks Surfacing"
        tags: [platform, devex, structural]
        must_have_ac: true
        description: >
          The template MUST surface tasks defined in specs/tasks.yaml via both
          CLI and HTTP so humans and agents can discover and track work items
          without reading raw YAML.
        adr: [ADR-0001]
        acceptance_criteria:
          - id: AC-TPL-TASKS-CLI
            text: >
              cargo xtask tasks-list prints all tasks with their IDs, titles,
              status, and requirement IDs (no filters implemented yet).
            tags: [future, recommended]
            must_have_ac: false
            note: "Optional CLI convenience; not kernel - CLAUDE.md says 'if implemented', runbooks/platform-kernel.md: 'not a kernel gate'"
            tests:
              - { type: bdd, tag: "@AC-TPL-TASKS-CLI", file: "specs/features/platform_tasks.feature" }

          - id: AC-TPL-TASKS-CREATE-CLI
            text: >
              cargo xtask task-create creates a new task in specs/tasks.yaml,
              validates that the requirement and ACs exist in spec_ledger.yaml,
              and rejects duplicate task IDs.
            tags: [future, recommended]
            must_have_ac: false
            note: "Optional CLI convenience; not kernel - part of future task management workflow"
            tests:
              - { type: bdd, tag: "@AC-TPL-TASKS-CREATE-CLI", file: "specs/features/platform_tasks.feature" }

          - id: AC-TPL-TASKS-UPDATE-CLI
            text: >
              cargo xtask task-update updates task fields (status, title, owner)
              in specs/tasks.yaml, enforces valid status transitions, and
              persists changes to the task definition.
            tags: [future, recommended]
            must_have_ac: false
            note: "Optional CLI convenience; not kernel - part of future task management workflow"
            tests:
              - { type: bdd, tag: "@AC-TPL-TASKS-UPDATE-CLI", file: "specs/features/platform_tasks.feature" }

          - id: AC-TPL-TASKS-HTTP
            text: >
              GET /platform/tasks returns a JSON representation of tasks.yaml,
              including id, title, requirement, acs, status, owner, labels,
              and docs fields, and supports status/requirement filters.
              POST /platform/tasks/{id}/status updates task status via HTTP
              (no POST /platform/tasks or PUT /platform/tasks/{id} implemented yet).
            tags: [future]
            must_have_ac: false
            tests:
              - { type: bdd, tag: "@AC-TPL-TASKS-HTTP", file: "specs/features/platform_tasks.feature" }

      - id: REQ-TPL-GRAPH-VISUALIZATION
        title: "Governance Graph Visualization"
        tags: [platform, structural, docs]
        must_have_ac: true
        description: >
          The template MUST provide a human-friendly visualization of the
          governance graph, so engineers can understand relationships
          between stories, requirements, ACs, and docs at a glance.
        adr: [ADR-0001]
        acceptance_criteria:
          - id: AC-TPL-GRAPH-MERMAID
            text: >
              cargo xtask graph-export --format mermaid emits a valid Mermaid
              graph (graph TD) that includes nodes for stories, requirements,
              and ACs, and edges showing their relationships.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "graph_export_mermaid", module: "graph::tests::graph_export_mermaid", file: "crates/spec-runtime/src/graph.rs" }
              - { type: bdd, tag: "@AC-TPL-GRAPH-MERMAID", file: "specs/features/graph_visualization.feature" }

      - id: REQ-TPL-CONFIG-INTEGRITY
        title: "Configuration is validated at startup"
        tags: [platform, config, structural]
        must_have_ac: true
        description: >
          Application startup MUST validate configuration against config_schema.yaml
          and fail fast with a clear error when invalid.
        acceptance_criteria:
          - id: AC-TPL-CONFIG-VALIDATION
            text: >
              On startup the service validates configuration against
              config_schema.yaml and exits non-zero with a clear validation
              error when the config is invalid.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-CONFIG-VALIDATION", file: "specs/features/config_validation.feature" }
              - { type: unit, tag: "config_validation_rejects_invalid", module: "config::tests::config_validation_rejects_invalid", file: "crates/spec-runtime/src/config.rs" }

      - id: REQ-TPL-IAC-ALIGNMENT
        title: "Infrastructure templates align with config schema"
        tags: [platform, ops, iac]
        must_have_ac: true
        description: >
          Sample IaC artifacts (docker-compose, k8s, terraform) MUST use the
          same ports and environment keys defined in config_schema.yaml and
          envs.yaml so examples stay trustworthy.
        acceptance_criteria:
          - id: AC-TPL-IAC-COMPOSE-ALIGN
            text: >
              docker-compose.yaml services (database, tracing, app) use the
              ports and env vars defined for the local environment in
              config_schema.yaml and envs.yaml.
            tags: [future]
            must_have_ac: false
            tests:
              - { type: unit, tag: "iac_compose_aligns_with_config", module: "local_docker::tests::iac_compose_aligns_with_config" }

          - id: AC-TPL-IAC-K8S-ALIGN
            text: >
              Kubernetes manifests under infra/k8s (Deployment/Service) use
              ports and env vars consistent with config_schema.yaml and the
              default environment.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "iac_k8s_aligns_with_config", module: "k8s_iac::tests::iac_k8s_aligns_with_config", file: "crates/spec-runtime/src/k8s_iac.rs" }

          - id: AC-TPL-IAC-TF-ALIGN
            text: >
              Terraform examples (when present) reference the same variables
              and defaults as config_schema.yaml for services and dependencies.
            tags: [future]
            must_have_ac: false
            tests:
              - { type: unit, tag: "iac_tf_aligns_with_config", module: "local_docker::tests::iac_tf_aligns_with_config", file: "crates/spec-runtime/src/local_docker.rs" }

      - id: REQ-TPL-LOCAL-RUNTIME
        title: "Local Runtime Sovereignty"
        tags: [platform, devex]
        must_have_ac: true
        description: >
          The template MUST provide a reproducible local runtime environment
          (Database, Observability) that works out-of-the-box with sensible defaults.
        adr: [ADR-0002]
        docs: [DESIGN-TPL-LOCAL-RUNTIME-001]
        acceptance_criteria:
          - id: AC-TPL-LOCAL-DOCKER
            text: >
              Optional local Docker compose (Postgres + Jaeger) aligns with the
              application's default local config; convenience only, not a gating
              kernel requirement.
            tags: [future]
            must_have_ac: false
            tests:
              - { type: unit, tag: "local_docker_compose_exists", module: "local_docker::tests::local_docker_compose_exists_and_has_core_services", file: "crates/spec-runtime/src/local_docker.rs" }

      - id: REQ-TPL-GOV-HOOKS
        title: "Governance Hooks"
        tags: [platform, devex, structural]
        must_have_ac: true
        description: >
          The template MUST provide a mechanism to install Git hooks that
          enforce governance checks (fmt, clippy, tests) before commit.
        adr: [ADR-0005]
        docs: [DESIGN-TPL-GOV-HOOKS-001]
        acceptance_criteria:
          - id: AC-TPL-HOOKS-INSTALL
            text: >
              The 'cargo xtask install-hooks' command creates a pre-commit hook
              that runs 'cargo run -p xtask -- precommit' inside the Nix devshell
              when available; failures are advisory and do not block commits.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: manual, tag: "git_commit_verify" }
              - { type: bdd, tag: "@AC-TPL-HOOKS-INSTALL", file: "specs/features/git_hooks.feature" }

      - id: REQ-TPL-AGENT-INTERFACE
        title: "Agent-Native Interface"
        tags: [platform, ai, structural]
        must_have_ac: true
        description: >
          The template MUST provide structured Skills for AI agents
          to execute governance workflows autonomously without guessing.
        adr: [ADR-0004]
        docs: [DESIGN-TPL-AGENT-INTERFACE-001]
        acceptance_criteria:
          - id: AC-TPL-AGENT-SKILLS
            text: >
              The .claude/skills directory contains executable skill definitions
              for feature development, release, and maintenance workflows,
              each referencing the appropriate xtask commands and platform APIs.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: manual, tag: "agent_skill_discovery" }
              - { type: bdd, tag: "@AC-TPL-AGENT-SKILLS", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-AGENT-HINTS
            text: >
              GET /platform/agent/hints returns prioritized task suggestions
              for agents, filtering tasks by Todo/InProgress status.
              Each hint includes: task_id, status, requirement_ids, ac_ids,
              reason, and recommended_sequence (array of commands/edits).
            tags: []
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-AGENT-HINTS" }

      - id: REQ-TPL-SKILLS-GUIDE
        title: "Agent Skills documentation and governance"
        tags: [platform, agent, docs]
        must_have_ac: true
        description: >
          The template MUST provide repo-specific guidance for Agent Skills that
          align with DevEx flows and governance, so Skills map to workflows
          (feature dev, maintenance, release) rather than individual commands.
        adr: [ADR-0004]
        docs: [DESIGN-TPL-SKILLS-GUIDE-001]
        acceptance_criteria:
          - id: AC-TPL-SKILLS-GUIDE-001
            text: >
              docs/AGENT_SKILLS.md exists and documents the recommended Skill set,
              SKILL.md templates, and best practices for this repo.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: docs, tag: "skills_guide_exists" }
              - { type: bdd, tag: "@AC-TPL-SKILLS-GUIDE-001", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-SKILLS-ALIGN-001
            text: >
              Existing .claude/skills/* are aligned with documented workflows
              (bootstrap-dev-env, governed-feature-dev, governed-maintenance,
              governed-release, governed-governance-debug).
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: manual, tag: "skills_alignment_review" }
              - { type: bdd, tag: "@AC-TPL-SKILLS-ALIGN-001", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-SKILLS-TOOLING
        title: "Rust-native Skills formatting and linting"
        tags: [platform, agent, devex]
        must_have_ac: true
        description: >
          Agent Skills definitions MUST be checked and normalized by Rust-native
          xtask commands to ensure consistency, avoid drift, and eliminate external
          scripting dependencies.
        adr: [ADR-0004, ADR-0005]
        docs: [DESIGN-TPL-SKILLS-TOOLING-001]
        acceptance_criteria:
          - id: AC-TPL-SKILLS-FMT
            text: >
              `cargo run -p xtask -- skills-fmt` normalizes SKILL.md files
              according to repo conventions (frontmatter, headings, links).
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-TPL-SKILLS-FMT", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-SKILLS-LINT
            text: >
              `cargo run -p xtask -- skills-lint` validates Skills frontmatter and
              content (name/description rules, references to flows and APIs).
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-TPL-SKILLS-LINT", file: "specs/features/xtask_devex.feature" }

      - id: REQ-TPL-SKILLS-GOVERNANCE
        title: "Agent Skills are governed artifacts with REQ/AC traceability"
        tags: [platform, agent, governance, devex]
        must_have_ac: true
        description: >
          Agent Skills are not ad-hoc documentation but governed artifacts with
          explicit requirements, acceptance criteria, and lifecycle management.
          Each Skill MUST map to a flow in devex_flows.yaml and have REQ/AC
          in spec_ledger.yaml, preventing Skill explosion and ensuring discovery.
        adr: [ADR-0020, ADR-0003, ADR-0005]
        docs: [DESIGN-TPL-SKILLS-GOVERNANCE-001]
        acceptance_criteria:
          - id: AC-TPL-SKILLS-GOVERNANCE-001
            text: >
              docs/SKILLS_GOVERNANCE.md exists and documents the governance spec,
              lifecycle (create/maintain/retire), validation rules, and ADR-0020
              principles for this repo.
            tags: [template, governance]
            must_have_ac: false
            note: "Documentation AC – part of governance guide."
            tests:
              - { type: ci, tag: "skills_governance_documented", note: "Governance documentation" }

          - id: AC-TPL-SKILLS-GOVERNANCE-002
            text: >
              Each Skill in .claude/skills/* has a corresponding REQ in spec_ledger.yaml
              and at least one AC defining its SKILL.md structure requirements.
            tags: [template, governance]
            must_have_ac: false
            note: "Guidance AC – validated manually during code review."
            tests:
              - { type: ci, tag: "skills_governance_guidance", note: "Governance guidance" }

          - id: AC-TPL-SKILLS-GOVERNANCE-003
            text: >
              docs/SKILLS_TEMPLATE.md exists and provides a copy-paste template
              for creating new Skills with checklist for name format, description
              quality (what + when), allowed-tools safety, and references.
            tags: [template, governance]
            must_have_ac: false
            note: "Documentation AC – part of governance guide."
            tests:
              - { type: ci, tag: "skills_template_documented", note: "Template documentation" }

          - id: AC-TPL-SKILLS-NAME-FORMAT
            text: >
              Skill names MUST be kebab-case, contain only lowercase letters/digits/hyphens,
              max 64 characters, and be unique within the project. skills-lint enforces this.
            tags: [template, governance]
            must_have_ac: false
            note: "Validation AC – enforced by skills-lint command."
            tests:
              - { type: ci, tag: "skills_name_format_enforced", note: "Governance enforcement" }

          - id: AC-TPL-SKILLS-DESCRIPTION-QUALITY
            text: >
              Skill descriptions MUST include both WHAT (capability) and WHEN (triggers/context),
              use third-person voice, and max 1024 characters.
              skills-lint must warn if description omits "when to use" triggers.
            tags: [template, governance]
            must_have_ac: false
            note: "Guidance AC – skills-lint provides warnings, not hard enforcement."
            tests:
              - { type: ci, tag: "skills_description_quality", note: "Governance guidance" }

          - id: AC-TPL-SKILLS-ALLOWED-TOOLS-SAFETY
            text: >
              Skill allowed-tools MUST follow least-privilege principle. skills-lint must warn if
              read-only Skill includes Write/Edit, or if unscoped Bash is used without justification.
              No hardcoded secrets in SKILL.md or supporting files.
            tags: [template, governance, security]
            must_have_ac: false
            note: "Guidance AC – skills-lint provides warnings on tool grants, not hard rejection."
            tests:
              - { type: ci, tag: "skills_tools_safety", note: "Governance guidance" }

          - id: AC-TPL-SKILLS-FLOW-MAPPING
            text: >
              Skill descriptions MUST reference at least one devex_flows entry or xtask command.
              Anti-pattern detection: warn if Skill name suggests it wraps single command (e.g., skill-check)
              instead of workflow. Lint rule: description must mention workflow scope.
            tags: [template, governance]
            must_have_ac: false
            note: "Guidance AC – skills-lint provides warnings on anti-patterns, not hard rejection."
            tests:
              - { type: ci, tag: "skills_flow_mapping", note: "Governance guidance" }

          - id: AC-TPL-SKILLS-LIFECYCLE-DOCS
            text: >
              docs/SKILLS_GOVERNANCE.md documents the full lifecycle: how to create a Skill
              (REQ + AC → Task → implement), maintain (keep devex_flows.yaml in sync), and retire
              (mark REQ deprecated, archive SKILL.md).
            tags: [template, governance]
            must_have_ac: false
            note: "Documentation AC - part of governance guide."
            tests:
              - { type: ci, tag: "skills_lifecycle_documented", note: "Governance documentation" }

      - id: REQ-TPL-AGENTS-GOVERNANCE
        title: "Claude Code agents are governed artifacts with REQ/AC traceability"
        tags: [platform, agent, governance, devex]
        must_have_ac: true
        description: >
          Claude Code subagents (.claude/agents/*.md) are not ad-hoc prompts but governed
          artifacts with explicit requirements, acceptance criteria, and lifecycle management.
          Each project agent MUST have REQ/AC coverage in spec_ledger.yaml, follow naming,
          description, and tool/permission policies, and be validated by agents-lint.
        adr: [ADR-0021, ADR-0003, ADR-0005]
        docs: [DESIGN-TPL-AGENTS-GOVERNANCE-001]
        acceptance_criteria:
          - id: AC-TPL-AGENTS-GOVERNANCE-001
            text: >
              docs/AGENTS_GOVERNANCE.md exists and documents the agent governance spec,
              lifecycle (create/maintain/retire), validation rules, and ADR-0021 principles
              for this repo.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: docs, tag: "agents_governance_doc_exists" }
              - { type: bdd, tag: "@AC-TPL-AGENTS-GOVERNANCE-001", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-AGENTS-GOVERNANCE-002
            text: >
              Each project agent in .claude/agents/* has a corresponding REQ in
              spec_ledger.yaml and at least one AC defining its configuration and
              system prompt requirements.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: manual, tag: "agents_req_ac_alignment" }
              - { type: bdd, tag: "@AC-TPL-AGENTS-GOVERNANCE-002", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-AGENTS-TEMPLATE-DOC
            text: >
              docs/AGENTS_TEMPLATE.md exists and provides a copy-paste template
              for creating new agents with checklist for name format, description
              quality (what + when), tools/permissionMode safety, model selection,
              and skills references.
            tags: [template]
            must_have_ac: true
            tests:
              - { type: docs, tag: "agents_template_doc_exists" }
              - { type: bdd, tag: "@AC-TPL-AGENTS-TEMPLATE-DOC", file: "specs/features/xtask_devex.feature" }

          - id: AC-TPL-AGENTS-NAME-FORMAT
            text: >
              Agent names MUST be kebab-case, contain only lowercase letters/digits/hyphens,
              max 64 characters, and be unique within the project. agents-lint enforces this.
            tags: [template, governance]
            must_have_ac: false
            note: "Validation AC – enforced by agents-lint command."
            tests:
              - { type: ci, tag: "agents_name_format_enforced", note: "Governance enforcement" }

          - id: AC-TPL-AGENTS-DESCRIPTION-QUALITY
            text: >
              Agent descriptions MUST include both WHAT (capability) and WHEN (triggers/context),
              and be ≤1024 characters. agents-lint MUST treat absence/emptiness as an error and
              missing WHEN as a warning.
            tags: [template, governance]
            must_have_ac: false
            note: "Guidance AC – agents-lint provides warnings, not hard enforcement."
            tests:
              - { type: ci, tag: "agents_description_quality", note: "Governance guidance" }

          - id: AC-TPL-AGENTS-TOOLS-PERMISSION-SAFETY
            text: >
              Agent tools and permissionMode MUST follow least-privilege and explicitness principles.
              agents-lint MUST error on invalid permissionMode values and warn on broad tool grants
              (e.g., full Bash + Edit + Write) without justification.
            tags: [template, governance, security]
            must_have_ac: false
            note: "Guidance AC – agents-lint provides warnings on tool grants, not hard rejection."
            tests:
              - { type: ci, tag: "agents_tools_safety", note: "Governance guidance" }

          - id: AC-TPL-AGENTS-MODEL-POLICY
            text: >
              Agent model selection MUST respect repo policy (e.g., default to 'inherit' or
              approved aliases only). agents-lint MUST error on unknown models and warn when
              expensive models (opus) are used without justification.
            tags: [template, governance, cost]
            must_have_ac: false
            note: "Guidance AC – agents-lint provides warnings on expensive models, not hard rejection."
            tests:
              - { type: ci, tag: "agents_model_policy", note: "Governance guidance" }

          - id: AC-TPL-AGENTS-SKILLS-REFERENCES
            text: >
              Agent 'skills' entries MUST reference existing Skills in .claude/skills/* or
              be omitted. agents-lint MUST error if an agent declares a Skill that does not exist.
            tags: [template, governance]
            must_have_ac: false
            note: "Validation AC – enforced by agents-lint command."
            tests:
              - { type: ci, tag: "agents_skills_references", note: "Governance enforcement" }

          - id: AC-TPL-AGENTS-LIFECYCLE-DOCS
            text: >
              docs/AGENTS_GOVERNANCE.md documents the full lifecycle: how to create an agent
              (REQ + AC → Task → implement), maintain (keep config in sync with flows and Skills),
              and retire (mark REQ deprecated, archive agent file).
            tags: [template, governance]
            must_have_ac: false
            note: "Documentation AC - part of governance guide."
            tests:
              - { type: ci, tag: "agents_lifecycle_documented", note: "Governance documentation" }

      - id: REQ-TPL-BUNDLE-CONTRACT
        title: "LLM bundle structure"
        tags: [platform, devex, ai]
        must_have_ac: true
        description: >
          The template MUST provide a predictable, governed structure for
          context bundles generated for agents. Bundles are first-class artifacts
          with explicit contracts: manifest structure, scope boundaries, and
          versioning. Agents and humans rely on bundles to understand task scope
          and dependencies.
        acceptance_criteria:
          - id: AC-TPL-BUNDLE-LAYOUT
            text: >
              `cargo xtask bundle <TASK>` creates `bundle/<TASK>/` with:
              (1) `bundle.yaml` manifest listing task_id, requirement_ids, ac_ids,
              referenced spec_ledger sections, docs, and tests;
              (2) `context.md` with bundled file contents (markdown format);
              (3) manifest includes bundle_version, git_sha, and timestamp for
              reproducibility.
            tags: [kernel, template, devex]
            must_have_ac: true
            note: "Bundle layout is a kernel contract - agents and CI depend on predictable structure."
            tests:
              - { type: bdd, tag: "@AC-TPL-BUNDLE-LAYOUT", file: "specs/features/bundles.feature" }

          - id: AC-TPL-BUNDLE-MANIFEST
            text: >
              `bundle.yaml` contains bundle_version (current: 1), task_id, requirement_ids,
              ac_ids, spec sections (with file paths and line anchors), referenced docs
              (with paths), and test handles (type, tag, file). Manifest is machine-readable
              and governs bundle scope boundaries.
            tags: [template, devex]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-BUNDLE-MANIFEST", file: "specs/features/bundles.feature" }

          - id: AC-TPL-BUNDLE-MINIMAL-SCOPE
            text: >
              Bundle scope is minimal: specs/ includes only referenced sections (not entire
              spec_ledger), docs/ includes only docs mentioned in ACs/tests, src/ is optional
              and narrowly scoped (crate-specific). Enforcement: bundle size audit in selftest
              catches over-inclusion.
            tags: [template, devex]
            must_have_ac: false
            note: "Minimal scope is aspirational (v3.4.0) but not enforced at bundle command time yet."
            tests:
              - { type: unit, tag: "bundle_scope_audit", file: "crates/xtask/src/commands/bundle.rs" }

  - id: US-TPL-PLATFORM-V3
    title: "The Recursive Platform"
    description: >
      The platform MUST be able to govern its own evolution by reading and
      writing its own specifications and task state, enabling autonomous
      loops where agents drive the development process.
    adr: [ADR-0001, ADR-0004]
    requirements:
      - id: REQ-TPL-GOV-WRITE-001
        title: "The system can persist task status changes to machine-managed state"
        tags: [platform, governance, write-layer]
        must_have_ac: true
        description: >
          The platform MUST provide a mechanism to update task status in a
          durable way that is reflected in the governance graph, without
          destroying human comments in the source YAML.
        adr: [ADR-0001]
        docs: [DESIGN-TPL-GOV-WRITE-001]
        acceptance_criteria:
          - id: AC-TPL-GOV-WRITE-TASK-STATUS-200
            text: "set_task_status writes durable state reflected in the governance graph."
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: integration, tag: "@AC-TPL-GOV-WRITE-TASK-STATUS-200" }

      - id: REQ-TPL-TASK-LIFECYCLE
        title: "Tasks must follow allowed status transitions"
        tags: [platform, governance, domain]
        must_have_ac: true
        description: >
          The platform MUST enforce valid state transitions for tasks (e.g. Todo -> InProgress)
          to prevent invalid workflows.
        adr: [ADR-0001]
        docs: [DESIGN-TPL-TASK-LIFECYCLE-001]
        acceptance_criteria:
          - id: AC-TPL-TASK-TRANSITIONS
            text: "Task status transitions are validated against the domain model."
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: unit, tag: "test_allowed_transitions", module: "governance::tests::test_allowed_transitions", file: "crates/business-core/src/lib.rs" }
              - { type: unit, tag: "test_forbidden_transitions", module: "governance::tests::test_forbidden_transitions", file: "crates/business-core/src/lib.rs" }

  - id: US-TPL-PHILOSOPHY-001
    title: "Template Philosophy and Kernel Contracts"
    description: >
      The template MUST make its opinions explicit, provide clear override paths,
      and support machine-native interfaces for AI and IDP consumption.
    adr: [ADR-0004, ADR-0005]
    requirements:

      - id: REQ-TPL-OPINIONATED-DEFAULTS
        title: "Opinionated defaults are intentional"
        tags: [kernel, philosophy]
        must_have_ac: true
        description: >
          The kernel MUST define a set of opinionated defaults for workflows,
          tooling, and platform behaviour, and treat them as first-class
          contracts, not hidden conventions.
        acceptance_criteria:
          - id: AC-TPL-OPINIONS-DOCUMENTED
            text: >
              docs/QUICKSTART.md and docs/ROADMAP.md include a 'Defaults &
              Opinions' section listing at least: environment model (Nix-first,
              Tier-1/Tier-2 split), CI gate (selftest as required), governance
              artifacts (questions, friction, forks), and agent surfaces
              (/platform/*, bundles, xtask CLI).
            tags: [template, philosophy]
            must_have_ac: false
            note: "Documentation AC - verified by BDD content inspection."
            tests:
              - { type: bdd, tag: "@AC-TPL-OPINIONS-DOCUMENTED", file: "specs/features/philosophy_docs.feature" }

      - id: REQ-TPL-OVERRIDE-PATH
        title: "Forks override via ACs, not hacks"
        tags: [kernel, philosophy]
        must_have_ac: true
        description: >
          Forks MUST be able to adjust template opinions by changing ACs and
          specs, rather than patching random code or CI in-place.
        acceptance_criteria:
          - id: AC-TPL-OVERRIDE-DOC
            text: >
              docs/how-to/change-template-opinion.md exists and describes the
              recommended override path: (1) Identify story/REQ/AC in spec_ledger.yaml,
              (2) Update AC text/tags/must_have_ac in the fork, (3) Update BDD scenarios,
              (4) Run selftest + ac-status, (5) Optionally log friction/question if
              the kernel made this hard.
            tags: [template, philosophy]
            must_have_ac: false
            note: "Documentation AC - verified by BDD content inspection."
            tests:
              - { type: bdd, tag: "@AC-TPL-OVERRIDE-DOC", file: "specs/features/philosophy_docs.feature" }
          - id: AC-TPL-OVERRIDE-TRACEABLE
            text: >
              specs/doc_index.yaml registers the override doc with tag 'override_path',
              and /platform/docs/index exposes it under a 'Kernel Overrides' category.
            tags: [template, philosophy]
            must_have_ac: false
            note: "Documentation AC - verified by BDD content inspection."
            tests:
              - { type: bdd, tag: "@AC-TPL-OVERRIDE-TRACEABLE", file: "specs/features/philosophy_docs.feature" }

      - id: REQ-TPL-AI-IDP-COMPAT
        title: "AI/IDP-native machine surfaces"
        tags: [kernel, philosophy, ai]
        must_have_ac: true
        description: >
          The kernel MUST support machine consumption of governance state via
          JSON-enabled CLIs and HTTP endpoints, enabling agents and IDPs to
          integrate without scraping text.
        acceptance_criteria:
          - id: AC-TPL-CLI-JSON-CORE
            text: >
              At minimum, cargo xtask ac-status, friction-list, questions-list,
              fork-list, and version accept --json and produce stable JSON output.
            tags: [kernel, philosophy, ai, idp]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-CLI-JSON-CORE", file: "specs/features/xtask_devex.feature" }
              - { type: unit, tag: "version_json_shape_is_stable",
                  module: "xtask::commands::version::tests::version_json_shape_is_stable",
                  file: "crates/xtask/src/commands/version.rs" }
              - { type: unit, tag: "ac_status_json_shape_is_stable",
                  module: "xtask::commands::ac_status::tests::ac_status_json_shape_is_stable",
                  file: "crates/xtask/src/commands/ac_status.rs" }
          - id: AC-TPL-PLATFORM-GOVERNANCE-APIS
            text: >
              /platform/questions, /platform/friction, and /platform/forks return
              JSON payloads that match their schemas and are linked from /platform/schema.
            tags: [kernel]
            must_have_ac: true
            tests:
              - { type: bdd, tag: "@AC-TPL-PLATFORM-GOVERNANCE-APIS", file: "specs/features/platform_introspection.feature" }

  # When adding your domain features:
  # - Copy the structure above (story → requirement → AC → tests)
  # - Use @AC-XXX tags in feature files
  # - Run: cargo run -p xtask -- ac-status
  # See docs/tutorials/first-ac-change.md for a walkthrough

```

# FILE: features/FT-TPL-CORE.yaml

```
id: FT-TPL-CORE
name: "Template Core Operations"
description: "Standard operational endpoints for service health monitoring and version information"
acceptance_criteria: [AC-TPL-001, AC-TPL-002]
owner: template
status: active
notes: "Keep and extend these capabilities in your service. Essential for production monitoring."

```

# FILE: flags/registry.yaml

```
flags:
  - key: refunds_v2
    description: "Enable v2 refund processing with improved fraud checks and async approval"
    owner: team-payments
    default: false
    expires_at: 2026-06-30

```

# FILE: flags/rollouts.yaml

```
environments:
  dev: { refunds_v2: 100 }
  staging: { refunds_v2: 50 }
  prod: { refunds_v2: 0 }

```

# FILE: crates/business-core/src/lib.rs

```
// Core business logic goes here
//
// This crate should contain:
// - Domain entities and business rules
// - Use case / application service logic
// - Port definitions (traits for adapters to implement)
//
// Architecture principles:
// - No dependencies on HTTP, database, or other adapters
// - Adapters (app-http, app-db, etc.) call core, never the reverse
// - Core defines ports (traits), adapters implement them
//

pub mod ports {
    use model::Task;

    /// Port for task persistence
    #[async_trait::async_trait]
    pub trait TaskRepository: Send + Sync {
        async fn save(&self, task: &Task) -> Result<(), String>;
        async fn find_by_id(&self, id: &str) -> Result<Option<Task>, String>;
        async fn find_all(&self) -> Result<Vec<Task>, String>;
        async fn update_status(
            &self,
            id: &str,
            status: model::TaskStatus,
        ) -> Result<Option<Task>, String>;
    }
}

pub mod use_cases {
    use super::ports::TaskRepository;
    use model::{Task, TaskStatus};

    /// Create a new task
    pub async fn create_task(repo: &dyn TaskRepository, title: String) -> Result<Task, String> {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        repo.save(&task).await?;
        Ok(task)
    }

    pub async fn get_task(repo: &dyn TaskRepository, id: String) -> Result<Option<Task>, String> {
        repo.find_by_id(&id).await
    }

    pub async fn list_tasks(repo: &dyn TaskRepository) -> Result<Vec<Task>, String> {
        repo.find_all().await
    }

    pub async fn update_task_status(
        repo: &dyn TaskRepository,
        id: String,
        status: TaskStatus,
    ) -> Result<Option<Task>, String> {
        repo.update_status(&id, status).await
    }
}

pub mod governance {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TaskStatus {
        Todo,
        InProgress,
        Review,
        Done,
    }

    impl TaskStatus {
        pub fn can_transition_to(&self, next: &TaskStatus) -> bool {
            use TaskStatus::*;
            match (self, next) {
                (Todo, InProgress) => true,
                (InProgress, Review) => true,
                (Review, Done) => true,
                (Review, InProgress) => true, // Backwards allowed
                (InProgress, Todo) => true,   // Backwards allowed
                _ => false,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Task {
        pub id: TaskId,
        pub title: String,
        pub status: TaskStatus,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct TaskId(pub String);

    #[derive(Debug, thiserror::Error)]
    pub enum GovernanceError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Serialization error: {0}")]
        Serialization(String),
        #[error("Task not found: {0:?}")]
        TaskNotFound(TaskId),
        #[error("Lock error: {0}")]
        Lock(String),
        #[error("Invalid transition from {from:?} to {to:?}")]
        InvalidTransition { from: TaskStatus, to: TaskStatus },
    }

    pub trait GovernanceRepository: Send + Sync {
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError>;
        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError>;
        fn set_task_status(
            &self,
            task_id: &TaskId,
            status: TaskStatus,
        ) -> Result<(), GovernanceError>;
    }

    impl GovernanceRepository for std::sync::Arc<dyn GovernanceRepository> {
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
            (**self).load_task(task_id)
        }

        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
            (**self).find_all_tasks()
        }

        fn set_task_status(
            &self,
            task_id: &TaskId,
            status: TaskStatus,
        ) -> Result<(), GovernanceError> {
            (**self).set_task_status(task_id, status)
        }
    }

    pub struct TaskService<R: GovernanceRepository> {
        repo: R,
    }

    impl<R: GovernanceRepository> TaskService<R> {
        pub fn new(repo: R) -> Self {
            Self { repo }
        }

        pub fn move_task(
            &self,
            id: &TaskId,
            new_status: TaskStatus,
        ) -> Result<(), GovernanceError> {
            let mut task = self.repo.load_task(id)?;
            if !task.status.can_transition_to(&new_status) {
                return Err(GovernanceError::InvalidTransition {
                    from: task.status,
                    to: new_status,
                });
            }
            task.status = new_status.clone();
            self.repo.set_task_status(id, new_status)?;
            Ok(())
        }

        pub fn list_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
            self.repo.find_all_tasks()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_allowed_transitions() {
            assert!(TaskStatus::Todo.can_transition_to(&TaskStatus::InProgress));
            assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Review));
            assert!(TaskStatus::Review.can_transition_to(&TaskStatus::Done));
            assert!(TaskStatus::Review.can_transition_to(&TaskStatus::InProgress));
        }

        #[test]
        fn test_forbidden_transitions() {
            assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::Todo));
            assert!(!TaskStatus::Todo.can_transition_to(&TaskStatus::Done));
        }
    }
}

```

# FILE: crates/app-http/src/agent.rs

```
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use business_core::governance::TaskService;
use serde::{Deserialize, Serialize};
use spec_runtime::hints::{self, HintEngine};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendedStep {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHint {
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub owner: String,
    pub labels: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub ac_ids: Vec<String>,
    pub reason: String,
    pub recommended_sequence: Vec<RecommendedStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHintsResponse {
    pub hints: Vec<AgentHint>,
}

#[derive(Debug, Deserialize)]
pub struct HintsFilters {
    pub owner: Option<String>,
    pub label: Option<String>,
    pub requirement: Option<String>,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new().route("/platform/agent/hints", get(agent_hints)).with_state(state)
}

async fn agent_hints(
    State(state): State<AppState>,
    Query(filters): Query<HintsFilters>,
) -> Result<Json<AgentHintsResponse>, crate::AppError> {
    let service = TaskService::new(state.governance_repo.clone());
    let tasks = service.list_tasks().map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to list tasks: {}", e),
        )
    })?;

    // Load full task definitions from tasks.yaml for rich metadata
    let tasks_path = state.workspace_root.join("specs/tasks.yaml");
    let task_definitions = adapters_spec_fs::tasks_def::load_tasks_definitions(&tasks_path)
        .map_err(|e| {
            crate::AppError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                crate::ErrorCode::InternalError,
                format!("Failed to load task definitions: {}", e),
            )
        })?;

    // Load AC coverage from feature_status.md
    let feature_status_path = state.workspace_root.join("docs/feature_status.md");
    let ac_index = hints::parse_feature_status(&feature_status_path).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to parse feature_status.md: {}", e),
        )
    })?;

    // Load devex_flows.yaml for flow-based command sequences
    let devex_path = state.workspace_root.join("specs/devex_flows.yaml");
    let devex_content = std::fs::read_to_string(&devex_path).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to read devex_flows.yaml: {}", e),
        )
    })?;
    let devex_spec: serde_yaml::Value = serde_yaml::from_str(&devex_content).map_err(|e| {
        crate::AppError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            crate::ErrorCode::InternalError,
            format!("Failed to parse devex_flows.yaml: {}", e),
        )
    })?;

    // Convert governance tasks to spec_runtime tasks
    let runtime_tasks: Vec<spec_runtime::Task> = tasks
        .iter()
        .filter_map(|t| {
            let task_id = t.id.0.clone();
            let definition = task_definitions.get(&task_id)?;

            Some(spec_runtime::Task {
                id: task_id,
                title: definition.title.clone(),
                status: format!("{:?}", t.status),
                requirement: definition.requirement.clone(),
                acs: definition.acs.clone(),
                labels: definition.labels.clone(),
                owner: definition.owner.clone(),
                docs: None,
                summary: definition.summary.clone().unwrap_or_default(),
                recommended_flows: definition.recommended_flows.clone(),
                depends_on: vec![],
            })
        })
        .collect();

    // Create HintEngine with AC coverage
    let engine = HintEngine::new(ac_index, runtime_tasks);
    let hint_engine_hints = engine.task_hints();

    // Convert HintEngine hints to AgentHints and build recommended sequences
    let mut hints: Vec<AgentHint> = hint_engine_hints
        .iter()
        .filter_map(|hint| {
            // Only include Todo and InProgress hints (HintEngine filters these)
            if !matches!(hint.status, hints::HintStatus::Open | hints::HintStatus::InProgress) {
                return None;
            }

            let task_id = match &hint.target {
                hints::HintTarget::Task { id } => id.clone(),
                _ => return None,
            };

            let definition = task_definitions.get(&task_id)?;

            // Build recommended sequence from recommended_flows
            let recommended_sequence = build_recommended_sequence(
                &task_id,
                &definition.recommended_flows,
                &definition.acs,
                &devex_spec,
            );

            Some(AgentHint {
                task_id,
                title: hint.title.clone(),
                status: match hint.status {
                    hints::HintStatus::Open => "open".to_string(),
                    hints::HintStatus::InProgress => "in_progress".to_string(),
                    hints::HintStatus::Done => "done".to_string(),
                },
                owner: definition.owner.clone().unwrap_or_else(|| "unassigned".to_string()),
                labels: hint.tags.clone(),
                requirement_ids: vec![definition.requirement.clone()],
                ac_ids: definition.acs.clone(),
                reason: hint.reason.details.clone(),
                recommended_sequence,
            })
        })
        .collect();

    // Apply filters
    hints.retain(|hint| {
        // Filter by owner
        if let Some(ref owner_filter) = filters.owner
            && !hint.owner.eq_ignore_ascii_case(owner_filter)
        {
            return false;
        }

        // Filter by label
        if let Some(ref label_filter) = filters.label
            && !hint.labels.iter().any(|l| l.eq_ignore_ascii_case(label_filter))
        {
            return false;
        }

        // Filter by requirement
        if let Some(ref req_filter) = filters.requirement
            && !hint.requirement_ids.iter().any(|r| r.eq_ignore_ascii_case(req_filter))
        {
            return false;
        }

        true
    });

    // Sort by: 1) status (in_progress before open), 2) priority label, 3) ID
    hints.sort_by(|a, b| {
        // Primary: status (in_progress before open)
        let status_order_a = if a.status == "in_progress" { 0 } else { 1 };
        let status_order_b = if b.status == "in_progress" { 0 } else { 1 };

        match status_order_a.cmp(&status_order_b) {
            std::cmp::Ordering::Equal => {
                // Secondary: priority label (high > medium > low > none)
                let priority_a = get_priority_order(&a.labels);
                let priority_b = get_priority_order(&b.labels);

                match priority_a.cmp(&priority_b) {
                    std::cmp::Ordering::Equal => {
                        // Tertiary: ID (alphabetical)
                        a.task_id.cmp(&b.task_id)
                    }
                    other => other,
                }
            }
            other => other,
        }
    });

    Ok(Json(AgentHintsResponse { hints }))
}

/// Helper function to determine priority order from labels
/// Returns 0 for highest priority (priority:high), higher numbers for lower priority
fn get_priority_order(labels: &[String]) -> u8 {
    for label in labels {
        let label_lower = label.to_ascii_lowercase();
        if label_lower == "priority:high" || label_lower == "high" {
            return 0;
        } else if label_lower == "priority:medium" || label_lower == "medium" {
            return 1;
        } else if label_lower == "priority:low" || label_lower == "low" {
            return 2;
        }
    }
    // No priority label = lowest priority
    3
}

/// Build recommended command sequence from task's recommended_flows
fn build_recommended_sequence(
    task_id: &str,
    recommended_flows: &[String],
    ac_ids: &[String],
    devex_spec: &serde_yaml::Value,
) -> Vec<RecommendedStep> {
    let mut sequence = Vec::new();

    // Extract flows map from devex_spec
    let flows = match devex_spec.get("flows") {
        Some(serde_yaml::Value::Mapping(m)) => m,
        _ => return sequence,
    };

    // Process each recommended flow
    for flow_name in recommended_flows {
        if let Some(flow_value) = flows.get(flow_name)
            && let Some(steps_value) = flow_value.get("steps")
            && let Some(steps_seq) = steps_value.as_sequence()
        {
            // Add each step as a command
            for step in steps_seq {
                if let Some(cmd) = step.as_str() {
                    let command_value = match cmd {
                        // Special handling for common commands with task-specific params
                        "bundle" => format!("cargo xtask bundle {}", task_id),
                        "test-ac" => {
                            if let Some(first_ac) = ac_ids.first() {
                                format!("cargo xtask test-ac {}", first_ac)
                            } else {
                                format!("cargo xtask {}", cmd)
                            }
                        }
                        "bdd" => "cargo xtask bdd".to_string(),
                        "selftest" => "cargo xtask selftest".to_string(),
                        "ac-new" => "cargo xtask ac-new".to_string(),
                        "adr-new" => "cargo xtask adr-new".to_string(),
                        "adr-check" => "cargo xtask adr-check".to_string(),
                        "audit" => "cargo xtask audit".to_string(),
                        "release-prepare" => "cargo xtask release-prepare".to_string(),
                        "release-verify" => "cargo xtask release-verify".to_string(),
                        _ => format!("cargo xtask {}", cmd),
                    };

                    sequence.push(RecommendedStep {
                        kind: "command".to_string(),
                        value: command_value,
                    });
                }
            }
        }
    }

    sequence
}

```

# FILE: crates/app-http/src/errors.rs

```
//! Error handling with observability and correlation
//!
//! This module provides a comprehensive error type that supports:
//! - Machine-readable error codes
//! - AC ID and Feature ID tracking (for product/feature correlation)
//! - Structured logging with correlation fields
//! - Proper HTTP responses with JSON error bodies
//! - Request ID correlation
//!
//! # Design Philosophy
//!
//! Errors should be:
//! 1. **Actionable**: Include enough context to debug issues
//! 2. **Structured**: Use typed fields instead of string concatenation
//! 3. **Correlated**: Include request ID, AC ID, feature ID for tracing
//! 4. **Secure**: Don't leak internal details to clients
//! 5. **Observable**: Log errors with structured data for analysis
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use crate::errors::{AppError, ErrorCode};
//!
//! // Simple error
//! return Err(AppError::bad_request("Invalid amount"));
//!
//! // Error with code and context
//! return Err(AppError::validation_error(
//!     ErrorCode::InvalidAmount,
//!     "Amount must be greater than 0"
//! ).with_context("amount_cents", payload.amount_cents));
//!
//! // Error with AC/Feature tracking
//! return Err(AppError::business_logic_error(
//!     ErrorCode::RefundNotEligible,
//!     "Order not eligible for refund"
//! ).with_ac_id("AC-123")
//!   .with_feature_id("FT-456"));
//! ```

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::{HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, warn};

use crate::middleware::request_id::RequestId;

/// Machine-readable error codes
///
/// These codes allow clients to programmatically handle different error scenarios
/// without parsing error messages. They also help with:
/// - Metrics aggregation (count errors by code)
/// - Alert rules (alert on specific error codes)
/// - Client-side error handling (show appropriate UI based on code)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors (4xx)
    InvalidRequest,
    InvalidAmount,
    MissingField,
    InvalidFormat,
    Unauthorized,

    // Business logic errors (4xx)
    RefundNotEligible,
    OrderNotFound,
    InsufficientFunds,
    DuplicateRequest,

    // System errors (5xx)
    InternalError,
    ServiceUnavailable,
    DatabaseError,
    ExternalServiceError,
    // Add more as needed for your domain
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InvalidRequest => write!(f, "INVALID_REQUEST"),
            ErrorCode::InvalidAmount => write!(f, "INVALID_AMOUNT"),
            ErrorCode::MissingField => write!(f, "MISSING_FIELD"),
            ErrorCode::InvalidFormat => write!(f, "INVALID_FORMAT"),
            ErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            ErrorCode::RefundNotEligible => write!(f, "REFUND_NOT_ELIGIBLE"),
            ErrorCode::OrderNotFound => write!(f, "ORDER_NOT_FOUND"),
            ErrorCode::InsufficientFunds => write!(f, "INSUFFICIENT_FUNDS"),
            ErrorCode::DuplicateRequest => write!(f, "DUPLICATE_REQUEST"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
            ErrorCode::ServiceUnavailable => write!(f, "SERVICE_UNAVAILABLE"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::ExternalServiceError => write!(f, "EXTERNAL_SERVICE_ERROR"),
        }
    }
}

/// Application error with full observability support
///
/// This error type includes:
/// - HTTP status code (for response)
/// - Error code (for clients and metrics)
/// - User message (safe to show to clients)
/// - Internal context (for logging, not shown to clients)
/// - AC ID and Feature ID (for product tracking)
/// - Request ID (for correlation - AC-TPL-004)
#[derive(Debug)]
pub struct AppError {
    /// HTTP status code to return
    status: StatusCode,
    /// Machine-readable error code
    code: ErrorCode,
    /// User-facing error message (safe to expose)
    message: String,
    /// Internal context for debugging (logged but not exposed to clients)
    context: HashMap<String, serde_json::Value>,
    /// AC (Acceptance Criteria) ID for tracking features
    ac_id: Option<String>,
    /// Feature ID for tracking which feature this relates to
    feature_id: Option<String>,
    /// Request ID for correlation (AC-TPL-004)
    /// If None, a new UUID will be generated when converting to response
    request_id: Option<String>,
}

impl AppError {
    /// Create a new error with status, code, and message
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            context: HashMap::new(),
            ac_id: None,
            feature_id: None,
            request_id: None,
        }
    }

    /// Create a bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::InvalidRequest, message)
    }

    /// Create a validation error (400)
    pub fn validation_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    /// Create a business logic error (422 Unprocessable Entity)
    pub fn business_logic_error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    /// Create a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::OrderNotFound, message)
    }

    /// Create an internal server error (500)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError, message)
    }

    /// Add context field for debugging
    ///
    /// Context is logged but not exposed to clients
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }

    /// Add AC (Acceptance Criteria) ID
    ///
    /// Used to track which acceptance criteria this error relates to
    pub fn with_ac_id(mut self, ac_id: impl Into<String>) -> Self {
        self.ac_id = Some(ac_id.into());
        self
    }

    /// Add Feature ID
    ///
    /// Used to track which feature this error relates to
    pub fn with_feature_id(mut self, feature_id: impl Into<String>) -> Self {
        self.feature_id = Some(feature_id.into());
        self
    }

    /// Add Request ID (AC-TPL-004)
    ///
    /// Used for distributed tracing and correlation.
    /// If not set, a UUID will be generated automatically.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Log the error with structured fields
    fn log_error(&self) {
        // Determine if this is a client error (4xx) or server error (5xx)
        let is_server_error = self.status.is_server_error();

        // Create structured log event
        if is_server_error {
            // Server errors are more severe - log as error level
            error!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.context,
                ac_id = ?self.ac_id,
                feature_id = ?self.feature_id,
                "Internal server error occurred"
            );
        } else {
            // Client errors are expected - log as warn level
            warn!(
                error_code = %self.code,
                status_code = %self.status.as_u16(),
                message = %self.message,
                context = ?self.context,
                ac_id = ?self.ac_id,
                feature_id = ?self.feature_id,
                "Client error occurred"
            );
        }
    }
}

/// JSON error response body
///
/// This is what clients receive when an error occurs.
/// Matches the ErrorResponse schema in openapi.yaml (AC-TPL-003).
#[derive(Debug, Serialize)]
struct ErrorResponse {
    /// Machine-readable error code (required by AC-TPL-003)
    error: String,
    /// Human-readable error message (required by AC-TPL-003)
    message: String,
    /// Request ID for correlation (required by AC-TPL-003, AC-TPL-004)
    #[serde(rename = "requestId")]
    request_id: String,
    /// Optional AC ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_id: Option<String>,
    /// Optional Feature ID (for debugging/tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    feature_id: Option<String>,
    // Note: context is NOT included (internal only)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error with full context
        self.log_error();

        // Get or generate request ID (AC-TPL-004)
        let request_id =
            self.request_id.clone().unwrap_or_else(|| RequestId::generate().to_string());

        // Create client-safe response matching ErrorResponse schema (AC-TPL-003)
        let body = Json(ErrorResponse {
            error: self.code.to_string(),
            message: self.message.clone(),
            request_id: request_id.clone(),
            ac_id: self.ac_id.clone(),
            feature_id: self.feature_id.clone(),
        });

        // Create response with status code
        let mut response = (self.status, body).into_response();

        // Add X-Request-ID header (AC-TPL-004)
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
        }

        response
    }
}

/// Convert JSON rejection errors to AppError
///
/// This allows us to handle JSON parsing errors consistently
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::validation_error(
            ErrorCode::InvalidRequest,
            format!("Invalid JSON: {}", rejection),
        )
    }
}

impl From<business_core::governance::GovernanceError> for AppError {
    fn from(error: business_core::governance::GovernanceError) -> Self {
        use business_core::governance::GovernanceError::*;
        match error {
            TaskNotFound(id) => AppError::not_found(format!("Task not found: {:?}", id)),
            InvalidTransition { from, to } => AppError::internal_error(format!(
                "Invalid status transition from {:?} to {:?}",
                from, to
            )),
            Lock(msg) => AppError::internal_error(format!("Lock error: {}", msg)),
            Io(e) => AppError::internal_error(format!("IO error: {}", e)),
            Serialization(msg) => AppError::internal_error(format!("Serialization error: {}", msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidAmount.to_string(), "INVALID_AMOUNT");
        assert_eq!(ErrorCode::RefundNotEligible.to_string(), "REFUND_NOT_ELIGIBLE");
    }

    #[test]
    fn test_bad_request_error() {
        let error = AppError::bad_request("Invalid input");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.message, "Invalid input");
    }

    #[test]
    fn test_error_with_context() {
        let error = AppError::validation_error(ErrorCode::InvalidAmount, "Amount must be positive")
            .with_context("amount", -100)
            .with_context("field", "amount_cents");

        assert!(error.context.contains_key("amount"));
        assert!(error.context.contains_key("field"));
    }

    #[test]
    fn test_error_with_ac_and_feature() {
        let error =
            AppError::business_logic_error(ErrorCode::RefundNotEligible, "Order not refundable")
                .with_ac_id("AC-123")
                .with_feature_id("FT-456");

        assert_eq!(error.ac_id, Some("AC-123".to_string()));
        assert_eq!(error.feature_id, Some("FT-456".to_string()));
    }

    #[test]
    fn test_error_serialization() {
        let error = AppError::validation_error(ErrorCode::InvalidAmount, "Amount must be positive")
            .with_ac_id("AC-123")
            .with_feature_id("FT-456")
            .with_request_id("req-test-123");

        let response = ErrorResponse {
            error: error.code.to_string(),
            message: error.message.clone(),
            request_id: error.request_id.clone().unwrap_or_default(),
            ac_id: error.ac_id.clone(),
            feature_id: error.feature_id.clone(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("INVALID_AMOUNT"));
        assert!(json.contains("Amount must be positive"));
        assert!(json.contains("req-test-123"));
        assert!(json.contains("AC-123"));
        assert!(json.contains("FT-456"));
        // Verify it uses "error" not "code" (AC-TPL-003)
        assert!(json.contains(r#""error":"INVALID_AMOUNT""#));
        // Verify it uses "requestId" not "request_id" (AC-TPL-003)
        assert!(json.contains(r#""requestId":"req-test-123""#));
    }

    #[test]
    fn test_governance_invalid_transition_maps_to_server_error() {
        use business_core::governance::TaskStatus;

        let app_error: AppError = business_core::governance::GovernanceError::InvalidTransition {
            from: TaskStatus::Todo,
            to: TaskStatus::Done,
        }
        .into();

        assert_eq!(app_error.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            app_error.message.contains("Invalid status transition"),
            "message should mention status transition, got: {}",
            app_error.message
        );
    }
}

```

# FILE: crates/app-http/src/lib.rs

```
use axum::{
    Router,
    extract::{Extension, Json},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

// Public modules
pub mod agent;
pub mod errors;
pub mod metrics;
pub mod middleware;
pub mod platform;
pub mod security;
pub mod tasks;

// Re-export commonly used types
pub use errors::{AppError, ErrorCode};
pub use middleware::{REQUEST_ID_HEADER, RequestId};

use business_core::governance::GovernanceRepository;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub governance_repo: Arc<dyn GovernanceRepository>,
    pub workspace_root: PathBuf,
    pub config: Option<spec_runtime::ValidatedConfig>,
    pub platform_auth: security::PlatformAuthConfig,
}

impl AppState {
    #[allow(dead_code)]
    fn new(governance_repo: Arc<dyn GovernanceRepository>) -> Self {
        let workspace_root = resolve_workspace_root();
        Self::with_config(governance_repo, workspace_root, None)
    }

    pub fn with_config(
        governance_repo: Arc<dyn GovernanceRepository>,
        workspace_root: PathBuf,
        config: Option<spec_runtime::ValidatedConfig>,
    ) -> Self {
        let config = config.or_else(|| load_validated_config(&workspace_root));
        let platform_auth = security::PlatformAuthConfig::from_sources(config.as_ref());
        platform_auth.warn_if_misconfigured();

        Self { governance_repo, workspace_root, config, platform_auth }
    }
}

/// Create the application router (reusable for both main and tests)
pub fn app(governance_repo: Arc<dyn GovernanceRepository>) -> Router {
    let workspace_root = resolve_workspace_root();
    let config = load_validated_config(&workspace_root);
    let app_state = AppState::with_config(governance_repo, workspace_root, config);
    build_router(app_state)
}

/// Create the application router with an explicit workspace root.
/// Useful for tests to avoid reliance on global environment variables.
pub fn app_with_workspace_root(
    governance_repo: Arc<dyn GovernanceRepository>,
    workspace_root: PathBuf,
) -> Router {
    let config = load_validated_config(&workspace_root);
    build_router(AppState::with_config(governance_repo, workspace_root, config))
}

/// Create an application router from an already-constructed state (e.g., when main has validated config).
pub fn app_with_state(app_state: AppState) -> Router {
    build_router(app_state)
}

fn build_router(app_state: AppState) -> Router {
    let auth_state = app_state.clone();
    let platform_state = app_state.clone();
    let platform_router = Router::new()
        .with_state(platform_state.clone())
        .merge(platform::router(platform_state.clone()))
        .route("/tasks/{id}/status", post(tasks::update_task_status))
        .layer(axum::middleware::from_fn_with_state(auth_state, middleware::platform_auth_guard))
        .with_state(platform_state.clone());

    let tasks_router =
        Router::new().with_state(app_state.clone()).route("/ui/tasks", get(tasks::tasks_ui));

    let agent_router = agent::router(app_state.clone());

    Router::new()
        // Template core endpoints - keep these
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/echo", post(echo)) // For demonstrating error handling in tests
        // Platform introspection endpoints
        .nest("/platform", platform_router)
        // Platform UI routes (at root level)
        .merge(platform::ui_router(platform_state))
        // Merge domain endpoints
        .merge(tasks_router)
        .merge(agent_router)
        // Middleware layers (applied in reverse order - bottom to top)
        .layer(axum::middleware::from_fn(metrics::metrics_middleware))
        .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        .layer(
            // Configure TraceLayer to include request_id field
            TraceLayer::new_for_http().make_span_with(|request: &axum::extract::Request| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = tracing::field::Empty, // Will be filled by request_id middleware
                )
            }),
        )
        .with_state(app_state)
}

// ============================================================================
// Handlers - showing edge -> core path
// ============================================================================

// ============================================================================
// Template Core Handlers - Keep these in your service
// ============================================================================

/// Health check endpoint
///
/// Demonstrates:
/// - Accessing request ID from extensions
/// - Basic instrumentation
/// - Simple JSON response
#[instrument(skip(_request_id))]
async fn health(Extension(_request_id): Extension<RequestId>) -> impl IntoResponse {
    // Log with request_id automatically included from span
    info!("Health check requested");

    // METRICS STUB: Increment health check counter
    // metrics::counter!("health_checks_total").increment(1);

    Json(HealthResponse { status: "ok".to_string(), service: "service-api".to_string() })
}

/// Version information endpoint
#[instrument]
async fn version() -> impl IntoResponse {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

/// Echo endpoint - Used for testing error handling
///
/// Demonstrates:
/// - Validation errors with error codes
/// - Request ID propagation through error responses
/// - ErrorResponse envelope structure
#[instrument(skip(request_id, payload))]
async fn echo(
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<EchoRequest>,
) -> Result<Json<EchoResponse>, AppError> {
    info!("Echo request received");

    // Validation: message cannot be empty
    if payload.message.is_empty() {
        return Err(AppError::validation_error(ErrorCode::MissingField, "Message cannot be empty")
            .with_context("field", "message")
            .with_ac_id("AC-TPL-003") // Links to error envelope AC
            .with_request_id(request_id.as_str())); // AC-TPL-004: Propagate request ID
    }

    Ok(Json(EchoResponse { message: payload.message }))
}

// ============================================================================
// Your Domain Handlers Go Here
// ============================================================================
//
// Example structure for adding a domain handler:
//
// #[instrument(skip(request_id, payload), fields(entity_id = %payload.id))]
// async fn create_entity(
//     Extension(request_id): Extension<RequestId>,
//     Json(payload): Json<CreateEntityRequest>,
// ) -> Result<(StatusCode, Json<EntityResponse>), AppError> {
//     info!("Processing entity creation");
//
//     // Validation
//     if payload.name.is_empty() {
//         return Err(AppError::validation_error(ErrorCode::MissingField, "Name required")
//             .with_context("field", "name")
//             .with_ac_id("AC-XXX")
//             .with_request_id(request_id.as_str()));
//     }
//
//     // Call core domain logic
//     let entity = core::entities::create(payload)?;
//
//     info!(entity_id = %entity.id, "Entity created");
//     Ok((StatusCode::CREATED, Json(entity.into())))
// }
//
// See docs/tutorials/first-ac-change.md for a complete walkthrough.

// ============================================================================
// DTOs - Request/Response types for HTTP boundary
// ============================================================================

// Template Core DTOs
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Debug, Serialize)]
struct VersionInfo {
    version: String,
    #[serde(rename = "gitSha")]
    git_sha: String,
}

// Echo endpoint DTOs (used for testing error handling)
#[derive(Debug, Deserialize)]
struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    pub message: String,
}

// Your Domain DTOs Go Here
// Example:
// #[derive(Debug, Deserialize)]
// pub struct CreateEntityRequest {
//     pub name: String,
//     #[serde(rename = "someField")]
//     pub some_field: String,
// }

// ============================================================================
// Error handling - See errors.rs for comprehensive error handling
// ============================================================================
//
// The old inline AppError enum has been replaced with a full-featured
// error type in errors.rs that provides:
// - Machine-readable error codes
// - AC ID and Feature ID tracking
// - Structured logging with correlation
// - Proper JSON error responses with context
//
// See errors.rs for implementation details and examples.

// ============================================================================
// Architecture Notes:
//
// This demonstrates hexagonal/clean architecture:
//
// 1. HTTP layer (this file):
//    - Handles HTTP concerns (routing, serialization, status codes)
//    - Translates HTTP requests -> domain operations
//    - Translates domain errors -> HTTP responses
//
// 2. Domain layer (crates/core):
//    - Pure business logic, no HTTP knowledge
//    - Called BY adapters, never calls adapters
//
// 3. Model layer (crates/model):
//    - Domain entities and value objects
//    - Shared across adapters and core
//
// 4. Telemetry (crates/telemetry):
//    - Cross-cutting concern for observability
//    - Initialized once at startup
//
// Key pattern: The dependency arrow points INWARD
//   app-http -> core  ([OK] correct)
//   core -> app-http  ([X] never!)
// ============================================================================

pub fn resolve_workspace_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

fn load_validated_config(workspace_root: &Path) -> Option<spec_runtime::ValidatedConfig> {
    let config_path = workspace_root.join("config/local.yaml");
    let schema_path = workspace_root.join("specs/config_schema.yaml");

    match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(cfg) => Some(cfg),
        Err(err) => {
            tracing::warn!(
                "Failed to validate config at {} against {}: {}",
                config_path.display(),
                schema_path.display(),
                err
            );
            None
        }
    }
}

```

# FILE: crates/app-http/src/main.rs

```
use app_http::{AppState, app_with_state, resolve_workspace_root};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry (tracing)
    telemetry::init_tracing("app-http");

    info!("Starting HTTP service");

    let workspace_root = resolve_workspace_root();

    let config_path = workspace_root.join("config/local.yaml");
    let schema_path = workspace_root.join("specs/config_schema.yaml");

    let validated_config = match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!(
                "Configuration error (config: {}, schema: {}): {}",
                config_path.display(),
                schema_path.display(),
                err
            );
            std::process::exit(1);
        }
    };

    // Initialize governance repository
    let specs_dir = workspace_root.join("specs");
    let governance_repo =
        std::sync::Arc::new(adapters_spec_fs::FsGovernanceRepository::new(specs_dir));

    // Build our application router from lib, reusing validated config
    let app_state = AppState::with_config(
        governance_repo,
        workspace_root.clone(),
        Some(validated_config.clone()),
    );
    let app = app_with_state(app_state);

    // Start server on the documented platform port
    let addr = SocketAddr::from(([0, 0, 0, 0], validated_config.http_port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

```

# FILE: crates/app-http/src/metrics.rs

```
/// Prometheus metrics for HTTP endpoints
///
/// This module provides:
/// - Global metrics registry
/// - HTTP request counter with labels (method, path, status)
/// - Middleware to automatically record requests
/// - `/metrics` endpoint handler
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};
use std::time::Instant;

/// Global Prometheus registry
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// HTTP requests total counter
///
/// Labels:
/// - `method`: HTTP method (GET, POST, etc.)
/// - `path`: Request path
/// - `status`: HTTP status code
static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    let opts = Opts::new("http_requests_total", "Total number of HTTP requests processed");
    let counter = IntCounterVec::new(opts, &["method", "path", "status"])
        .expect("Failed to create HTTP_REQUESTS_TOTAL metric");

    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("Failed to register HTTP_REQUESTS_TOTAL metric");

    counter
});

/// Metrics endpoint handler
///
/// Returns Prometheus metrics in text format
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            let metrics_text = String::from_utf8(buffer).unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to convert metrics to UTF-8");
                String::from("# Error: failed to encode metrics")
            });
            (StatusCode::OK, metrics_text).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response()
        }
    }
}

/// Middleware to record HTTP metrics
///
/// Records each request with method, path, and status labels
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status().as_u16().to_string();
    let elapsed = start.elapsed();

    // Record metrics
    HTTP_REQUESTS_TOTAL.with_label_values(&[&method, &path, &status]).inc();

    tracing::debug!(
        method = %method,
        path = %path,
        status = %status,
        elapsed_ms = elapsed.as_millis(),
        "HTTP request processed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counter_increments() {
        // Increment counter
        HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/test", "200"]).inc();

        // Verify it was recorded
        let metric_families = REGISTRY.gather();
        let http_requests = metric_families.iter().find(|mf| mf.name() == "http_requests_total");

        assert!(http_requests.is_some(), "http_requests_total metric should be registered");
    }

    #[tokio::test]
    async fn test_metrics_handler_returns_text() {
        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

```

# FILE: crates/app-http/src/middleware/mod.rs

```
//! HTTP middleware for cross-cutting concerns
//!
//! This module contains middleware for:
//! - Request ID correlation (distributed tracing)
//! - Future: Rate limiting, authentication, etc.

pub mod platform_auth;
pub mod request_id;

pub use platform_auth::{PLATFORM_AUTH_HEADER, platform_auth_guard};
pub use request_id::{REQUEST_ID_HEADER, RequestId, request_id_middleware};

```

# FILE: crates/app-http/src/middleware/platform_auth.rs

```
use axum::http::{Method, Request};
use axum::{body::Body, extract::State, http::StatusCode, middleware::Next, response::Response};

use crate::{AppError, AppState, ErrorCode};

pub const PLATFORM_AUTH_HEADER: &str = "x-platform-token";

/// Enforces platform auth for write endpoints when PLATFORM_AUTH_MODE=basic.
pub async fn platform_auth_guard(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !state.platform_auth.requires_auth() {
        return Ok(next.run(request).await);
    }

    if matches!(request.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(request).await);
    }

    let provided = request.headers().get(PLATFORM_AUTH_HEADER).and_then(|v| v.to_str().ok());

    if state.platform_auth.is_authorized(provided) {
        return Ok(next.run(request).await);
    }

    Err(AppError::new(
        StatusCode::UNAUTHORIZED,
        ErrorCode::Unauthorized,
        "Unauthorized: missing or invalid platform token",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode},
        routing::get,
    };
    use business_core::governance::{
        GovernanceError, GovernanceRepository, Task, TaskId, TaskStatus,
    };
    use std::{path::PathBuf, sync::Arc};
    use tower::ServiceExt;

    #[derive(Clone)]
    struct NoopRepo;

    impl GovernanceRepository for NoopRepo {
        fn load_task(&self, task_id: &TaskId) -> Result<Task, GovernanceError> {
            Err(GovernanceError::TaskNotFound(task_id.clone()))
        }

        fn find_all_tasks(&self) -> Result<Vec<Task>, GovernanceError> {
            Ok(vec![])
        }

        fn set_task_status(
            &self,
            _task_id: &TaskId,
            _status: TaskStatus,
        ) -> Result<(), GovernanceError> {
            Ok(())
        }
    }

    async fn protected_handler() -> &'static str {
        "ok"
    }

    fn app_state(mode: crate::security::PlatformAuthMode, token: Option<&str>) -> AppState {
        AppState {
            governance_repo: Arc::new(NoopRepo),
            workspace_root: PathBuf::new(),
            config: None,
            platform_auth: crate::security::PlatformAuthConfig {
                mode,
                token: token.map(|t| t.to_string()),
            },
        }
    }

    fn guarded_router(state: AppState) -> Router {
        Router::new()
            .route("/platform/protected", get(protected_handler).post(protected_handler))
            .layer(axum::middleware::from_fn_with_state(state.clone(), platform_auth_guard))
            .with_state(state)
    }

    #[tokio::test]
    async fn rejects_post_without_token_in_basic_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_post_with_correct_token() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/platform/protected")
            .header(PLATFORM_AUTH_HEADER, "secret")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn allows_get_without_auth_even_in_basic_mode() {
        let state = app_state(crate::security::PlatformAuthMode::Basic, Some("secret"));
        let app = guarded_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/platform/protected")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.expect("handler should respond");
        assert_eq!(response.status(), StatusCode::OK);
    }
}

```

# FILE: crates/app-http/src/middleware/request_id.rs

```
//! Request ID correlation middleware
//!
//! This middleware implements distributed tracing correlation by:
//! 1. Reading X-Request-ID header from incoming requests (if present)
//! 2. Generating a new UUID if no request ID is provided
//! 3. Storing the ID in request extensions for handler access
//! 4. Adding the ID to the tracing span for log correlation
//! 5. Including it in the response header for client tracking
//!
//! # Observability Story
//!
//! Request IDs enable:
//! - **Distributed Tracing**: Track a request across multiple services
//! - **Log Correlation**: Group all logs for a single request
//! - **Debugging**: Clients can provide request IDs when reporting issues
//! - **Metrics**: Correlate metrics with specific requests
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use axum::Router;
//! use crate::middleware::request_id::RequestIdLayer;
//!
//! let app = Router::new()
//!     .route("/api/endpoint", get(handler))
//!     .layer(RequestIdLayer);
//! ```
//!
//! # Handler Access
//!
//! ```rust,ignore
//! use axum::extract::Extension;
//! use crate::middleware::request_id::RequestId;
//!
//! async fn handler(Extension(request_id): Extension<RequestId>) -> impl IntoResponse {
//!     info!(request_id = %request_id, "Processing request");
//!     // ... handler logic
//! }
//! ```

use axum::{
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::Response,
};
use tracing::Span;
use uuid::Uuid;

/// Header name for request ID (standard practice)
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Typed wrapper for request ID
///
/// This newtype provides type safety and makes it clear when we're working
/// with request IDs vs arbitrary strings.
#[derive(Debug, Clone)]
pub struct RequestId(String);

impl RequestId {
    /// Create a new request ID from a string
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new random request ID
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the request ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Request ID middleware implementation
///
/// This is the core middleware function that:
/// 1. Extracts or generates a request ID
/// 2. Adds it to the tracing span
/// 3. Stores it in request extensions
/// 4. Adds it to the response headers
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Step 1: Extract request ID from header or generate a new one
    let request_id = extract_or_generate_request_id(&request);

    // Step 2: Record the request ID in the current tracing span
    // This ensures all logs within this request context include the request_id field
    Span::current().record("request_id", request_id.as_str());

    // Step 3: Store request ID in request extensions
    // This allows handlers to access the request ID via Extension<RequestId>
    request.extensions_mut().insert(request_id.clone());

    // Step 4: Process the request through the handler chain
    let mut response = next.run(request).await;

    // Step 5: Add request ID to response headers
    // This allows clients to correlate responses with their requests
    if let Ok(header_value) = HeaderValue::from_str(request_id.as_str()) {
        response.headers_mut().insert(HeaderName::from_static("x-request-id"), header_value);
    }

    response
}

/// Extract request ID from header or generate a new one
fn extract_or_generate_request_id(request: &Request) -> RequestId {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| RequestId::new(s.to_string()))
        .unwrap_or_else(RequestId::generate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_display() {
        let id = RequestId::new("test-123".to_string());
        assert_eq!(format!("{}", id), "test-123");
    }

    #[test]
    fn test_request_id_as_str() {
        let id = RequestId::new("test-456".to_string());
        assert_eq!(id.as_str(), "test-456");
    }

    #[test]
    fn test_request_id_generate() {
        let id1 = RequestId::generate();
        let id2 = RequestId::generate();
        // Generated IDs should be different
        assert_ne!(id1.as_str(), id2.as_str());
        // Should be valid UUIDs
        assert!(Uuid::parse_str(id1.as_str()).is_ok());
        assert!(Uuid::parse_str(id2.as_str()).is_ok());
    }

    #[test]
    fn test_extract_or_generate_from_empty_request() {
        use axum::body::Body;
        use axum::http::Request;

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .map_err(|_| ())
            .unwrap_or_else(|_| Request::builder().uri("/").body(Body::empty()).unwrap());

        let request_id = extract_or_generate_request_id(&request);
        // Should be a valid UUID since no header was provided
        assert!(Uuid::parse_str(request_id.as_str()).is_ok());
    }

    #[test]
    fn test_extract_or_generate_from_request_with_header() {
        use axum::body::Body;
        use axum::http::Request;

        let test_id = "test-request-id-12345";
        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, test_id)
            .body(Body::empty())
            .unwrap_or_else(|_| Request::builder().uri("/").body(Body::empty()).unwrap());

        let request_id = extract_or_generate_request_id(&request);
        assert_eq!(request_id.as_str(), test_id);
    }
}

```

# FILE: crates/app-http/src/platform.rs

```
use crate::{AppError, AppState, ErrorCode};
use adapters_spec_fs::tasks_state;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use business_core::governance::{TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use spec_runtime::{ValidatedConfig, load_all_specs, load_service_metadata};
use std::collections::HashMap;
use std::fs;

mod forks;
mod friction;
mod questions;
mod ui;

/// Platform API routes (mounted at /platform)
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // API routes
        .route("/graph", get(get_graph))
        .route("/schema", get(get_schema))
        .route("/schema/{name}", get(get_schema_by_name_handler))
        .route("/devex/flows", get(get_devex_flows))
        .route("/docs/index", get(get_docs_index))
        .route("/status", get(get_status))
        .route("/coverage", get(get_coverage))
        .route("/tasks", get(get_tasks))
        .route("/tasks/suggest-next", get(get_suggest_next))
        .route("/tasks/graph", get(get_task_graph))
        .merge(friction::router())
        .merge(questions::router())
        .merge(forks::router())
        .with_state(state)
}

/// UI routes (mounted at root)
pub fn ui_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(ui::dashboard))
        .route("/ui", get(ui::dashboard))
        .route("/ui/graph", get(ui::graph_view))
        .route("/ui/flows", get(ui::flows_view))
        .route("/ui/coverage", get(ui::coverage_view))
        .with_state(state)
}

#[derive(Deserialize)]
struct SuggestNextQuery {
    task: String,
}

async fn get_suggest_next(
    State(state): State<AppState>,
    Query(q): Query<SuggestNextQuery>,
) -> Json<spec_runtime::tasks::SuggestedSequence> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))
        .expect("Failed to load tasks.yaml");
    let devex_spec = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex_flows.yaml");
    let ledger = spec_runtime::load_spec_ledger(&root.join("specs/spec_ledger.yaml"))
        .expect("Failed to load spec_ledger.yaml");

    let suggestion =
        spec_runtime::tasks::suggest_next(root, &q.task, &tasks_spec, &devex_spec, &ledger)
            .expect("Failed to generate suggestion");

    Json(suggestion)
}

#[derive(Serialize)]
struct PlatformStatus {
    service: ServiceInfo,
    governance: GovernanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<ConfigSummary>,
}

#[derive(Serialize)]
struct ServiceInfo {
    service_id: String,
    template_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    links: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<String>,
}

#[derive(Serialize)]
struct GovernanceStatus {
    ledger: LedgerCounts,
    devex: DevExCounts,
    docs: DocCounts,
    tasks: TaskCounts,
    questions: QuestionCounts,
    friction: FrictionCounts,
    forks: ForkCounts,
    policies: PolicyStatus,
}

#[derive(Serialize)]
struct LedgerCounts {
    stories: usize,
    requirements: usize,
    acs: usize,
}

#[derive(Serialize)]
struct DevExCounts {
    commands: usize,
    flows: usize,
}

#[derive(Serialize)]
struct DocCounts {
    total: usize,
    design: usize,
}

#[derive(Serialize)]
struct TaskCounts {
    total: usize,
}

#[derive(Serialize)]
struct QuestionCounts {
    open: usize,
    answered: usize,
    resolved: usize,
    total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    top_open: Vec<QuestionSummary>,
}

#[derive(Serialize)]
struct QuestionSummary {
    id: String,
    summary: String,
    flow: String,
}

#[derive(Serialize)]
struct FrictionCounts {
    total: usize,
    open: usize,
    by_severity: SeverityCounts,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    recent: Vec<FrictionSummary>,
}

#[derive(Serialize)]
struct SeverityCounts {
    low: usize,
    medium: usize,
    high: usize,
    critical: usize,
}

#[derive(Serialize)]
struct FrictionSummary {
    id: String,
    date: String,
    severity: String,
    summary: String,
    category: String,
}

#[derive(Serialize)]
struct ForkCounts {
    total: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ids: Vec<String>,
}

#[derive(Serialize)]
struct PolicyStatus {
    status: String,
}

#[derive(Serialize, Clone)]
pub(crate) struct ConfigSummary {
    env: Option<String>,
    http_port: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    settings: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    secrets_redacted: HashMap<String, String>,
    auth: AuthSummary,
}

#[derive(Serialize, Clone)]
struct AuthSummary {
    mode: String,
    token_present: bool,
}

#[derive(Deserialize)]
struct PolicyStatusReport {
    summary: String,
}

pub(crate) fn config_summary(state: &AppState) -> Option<ConfigSummary> {
    let config = state.config.as_ref()?;
    Some(ConfigSummary::from_parts(config, &state.platform_auth))
}

impl ConfigSummary {
    fn from_parts(config: &ValidatedConfig, auth: &crate::security::PlatformAuthConfig) -> Self {
        let settings = settings_as_json(&config.settings);

        ConfigSummary {
            env: config.env.clone(),
            http_port: config.http_port,
            settings,
            secrets_redacted: redacted_secrets(&config.secrets),
            auth: AuthSummary {
                mode: auth.mode_label().to_string(),
                token_present: auth.token_present(),
            },
        }
    }
}

fn settings_as_json(
    source: &HashMap<String, serde_yaml::Value>,
) -> HashMap<String, serde_json::Value> {
    let mut out = HashMap::new();

    for (k, v) in source {
        if let Ok(json_val) = serde_json::to_value(v) {
            out.insert(k.clone(), json_val);
        }
    }

    out
}

fn redacted_secrets(secrets: &HashMap<String, String>) -> HashMap<String, String> {
    secrets.keys().map(|k| (k.clone(), "[REDACTED]".to_string())).collect()
}

async fn get_graph(State(state): State<AppState>) -> Json<spec_runtime::Graph> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).expect("Failed to load specs");
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)
        .expect("Failed to build graph");
    Json(graph)
}

async fn get_devex_flows(State(state): State<AppState>) -> Json<serde_json::Value> {
    let root = &state.workspace_root;
    let devex = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"))
        .expect("Failed to load devex flows");
    Json(serde_json::to_value(devex).unwrap())
}

async fn get_docs_index(State(state): State<AppState>) -> Json<serde_json::Value> {
    let root = &state.workspace_root;
    let docs = spec_runtime::load_doc_index(&root.join("specs/doc_index.yaml"))
        .expect("Failed to load doc index");
    Json(serde_json::to_value(docs).unwrap())
}

async fn get_status(State(state): State<AppState>) -> Json<PlatformStatus> {
    let root = &state.workspace_root;
    let specs = load_all_specs(root).expect("Failed to load specs");
    let tasks_spec =
        spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).expect("Failed to load tasks");

    let ledger_counts = LedgerCounts {
        stories: specs.ledger.stories.len(),
        requirements: specs.ledger.stories.iter().map(|s| s.requirements.len()).sum(),
        acs: specs
            .ledger
            .stories
            .iter()
            .flat_map(|s| s.requirements.iter())
            .map(|r| r.acceptance_criteria.len())
            .sum(),
    };

    let devex_counts =
        DevExCounts { commands: specs.devex.commands.len(), flows: specs.devex.flows.len() };

    let doc_counts = DocCounts {
        total: specs.docs.docs.len(),
        design: specs.docs.docs.iter().filter(|d| d.doc_type == "design_doc").count(),
    };

    let task_counts = TaskCounts { total: tasks_spec.tasks.len() };

    // Load question counts
    let question_counts = load_question_counts(root);

    // Load friction counts
    let friction_counts = load_friction_counts(root);

    // Load fork counts
    let fork_counts = load_fork_counts(root);

    // Read policy status from last policy-test run
    let policy_path = root.join("target/policy_status.json");
    let policy_status = if let Ok(content) = fs::read_to_string(policy_path) {
        serde_json::from_str::<PolicyStatusReport>(&content)
            .map(|r| r.summary)
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml"))
        .expect("Failed to load service_metadata.yaml");

    let template_version =
        metadata.template_version.clone().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    let service_info = ServiceInfo {
        service_id: metadata.service_id.clone(),
        template_version,
        display_name: metadata.display_name.clone(),
        description: metadata.description.clone(),
        links: metadata.links.clone(),
        tags: metadata.tags.clone(),
    };

    let config = config_summary(&state);

    Json(PlatformStatus {
        service: service_info,
        governance: GovernanceStatus {
            ledger: ledger_counts,
            devex: devex_counts,
            docs: doc_counts,
            tasks: task_counts,
            questions: question_counts,
            friction: friction_counts,
            forks: fork_counts,
            policies: PolicyStatus { status: policy_status },
        },
        config,
    })
}

/// Load question counts from questions/ directory
fn load_question_counts(root: &std::path::Path) -> QuestionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Question {
        id: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        status: String,
        context: QuestionContext,
    }

    #[derive(Deserialize)]
    struct QuestionContext {
        flow: String,
    }

    let questions_dir = root.join("questions");
    if !questions_dir.exists() {
        return QuestionCounts { open: 0, answered: 0, resolved: 0, total: 0, top_open: vec![] };
    }

    let mut open = 0;
    let mut answered = 0;
    let mut resolved = 0;
    let mut total = 0;
    let mut open_questions = Vec::new();

    if let Ok(entries) = fs::read_dir(&questions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(question) = serde_yaml::from_str::<Question>(&content)
            {
                total += 1;
                match question.status.as_str() {
                    "open" => {
                        open += 1;
                        open_questions.push(QuestionSummary {
                            id: question.id,
                            summary: question.summary,
                            flow: question.context.flow,
                        });
                    }
                    "answered" => answered += 1,
                    "resolved" => resolved += 1,
                    _ => {}
                }
            }
        }
    }

    // Take top 3 open questions
    open_questions.truncate(3);

    QuestionCounts { open, answered, resolved, total, top_open: open_questions }
}

/// Load friction counts from friction/ directory
fn load_friction_counts(root: &std::path::Path) -> FrictionCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct FrictionEntry {
        id: String,
        date: String,
        #[serde(default)]
        severity: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        category: String,
        #[serde(default)]
        status: String,
    }

    let friction_dir = root.join("friction");
    if !friction_dir.exists() {
        return FrictionCounts {
            total: 0,
            open: 0,
            by_severity: SeverityCounts { low: 0, medium: 0, high: 0, critical: 0 },
            recent: vec![],
        };
    }

    let mut total = 0;
    let mut open = 0;
    let mut by_severity = SeverityCounts { low: 0, medium: 0, high: 0, critical: 0 };
    let mut all_entries = Vec::new();

    if let Ok(entries) = fs::read_dir(&friction_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|s| s.to_str()) == Some("README.yaml") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(friction) = serde_yaml::from_str::<FrictionEntry>(&content)
            {
                total += 1;

                // Count open friction
                if friction.status == "open" || friction.status.is_empty() {
                    open += 1;
                }

                // Count by severity
                match friction.severity.as_str() {
                    "low" => by_severity.low += 1,
                    "medium" => by_severity.medium += 1,
                    "high" => by_severity.high += 1,
                    "critical" => by_severity.critical += 1,
                    _ => {}
                }

                all_entries.push(friction);
            }
        }
    }

    // Sort by date (most recent first) and take top 5
    all_entries.sort_by(|a, b| b.date.cmp(&a.date));
    let recent: Vec<FrictionSummary> = all_entries
        .into_iter()
        .take(5)
        .map(|e| FrictionSummary {
            id: e.id,
            date: e.date,
            severity: e.severity,
            summary: e.summary,
            category: e.category,
        })
        .collect();

    FrictionCounts { total, open, by_severity, recent }
}

/// Load fork counts from forks/fork_registry.yaml
fn load_fork_counts(root: &std::path::Path) -> ForkCounts {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ForkRegistry {
        #[serde(default)]
        forks: Vec<ForkEntry>,
    }

    #[derive(Deserialize)]
    struct ForkEntry {
        id: String,
    }

    let registry_path = root.join("forks/fork_registry.yaml");
    if !registry_path.exists() {
        return ForkCounts { total: 0, ids: vec![] };
    }

    if let Ok(content) = fs::read_to_string(&registry_path)
        && let Ok(registry) = serde_yaml::from_str::<ForkRegistry>(&content)
    {
        let ids: Vec<String> = registry.forks.iter().map(|f| f.id.clone()).collect();
        let total = ids.len();
        ForkCounts { total, ids }
    } else {
        ForkCounts { total: 0, ids: vec![] }
    }
}

#[derive(Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub req: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<TaskOut>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskOut {
    pub id: String,
    pub title: String,
    pub requirement: String,
    pub acs: Vec<String>,
    pub status: String,
    pub owner: Option<String>,
    pub labels: Vec<String>,
    pub docs: Option<TaskDocsOut>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskDocsOut {
    pub design: Vec<String>,
    pub plan: Vec<String>,
}

fn normalize_status(raw: &str) -> String {
    let key = raw.trim().to_ascii_lowercase().replace([' ', '-'], "_");

    match key.as_str() {
        "todo" | "open" => "Todo".to_string(),
        "inprogress" | "in_progress" => "InProgress".to_string(),
        "review" => "Review".to_string(),
        "done" | "closed" => "Done".to_string(),
        _ => {
            tracing::warn!(
                raw_status = raw,
                normalized_status = "Todo",
                "Unknown task status provided; defaulting to Todo"
            );
            "Todo".to_string()
        }
    }
}

async fn get_tasks(
    State(state): State<AppState>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<TasksResponse>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).map_err(|err| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to load specs/tasks.yaml: {}", err),
        )
    })?;

    let state_map =
        tasks_state::get_all_tasks(&root.join("specs/tasks_state.yaml")).map_err(|err| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                format!("Failed to load task state: {}", err),
            )
        })?;

    let tasks = tasks_spec
        .tasks
        .into_iter()
        .filter_map(|t| {
            let effective_status = state_map
                .get(&TaskId(t.id.clone()))
                .cloned()
                .map(task_status_to_string)
                .unwrap_or_else(|| normalize_status(&t.status));

            if filters.status.as_ref().is_some_and(|s| !effective_status.eq_ignore_ascii_case(s)) {
                return None;
            }

            if filters.req.as_ref().is_some_and(|r| t.requirement != *r) {
                return None;
            }

            let mut task_out: TaskOut = t.into();
            task_out.status = effective_status;

            Some(task_out)
        })
        .collect();

    Ok(Json(TasksResponse { tasks }))
}

impl From<spec_runtime::Task> for TaskOut {
    fn from(t: spec_runtime::Task) -> Self {
        TaskOut {
            id: t.id,
            title: t.title,
            requirement: t.requirement,
            acs: t.acs,
            status: t.status,
            owner: t.owner,
            labels: t.labels,
            docs: t.docs.map(|d| TaskDocsOut { design: d.design, plan: d.plan }),
        }
    }
}

#[derive(Serialize)]
pub struct CoverageSummary {
    pub passing: usize,
    pub failing: usize,
    pub unknown: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CoverageDetail {
    pub id: String,
    pub title: String,
    pub status: String,
    pub story: String,
    pub requirement: String,
    pub scenarios: Vec<String>,
}

#[derive(Serialize)]
pub struct CoverageResponse {
    pub summary: CoverageSummary,
    pub details: Vec<CoverageDetail>,
}

// Cucumber JSON format structures for parsing BDD output
#[derive(Debug, Deserialize)]
struct CucumberReport(Vec<CucumberFeature>);

#[derive(Debug, Deserialize)]
struct CucumberFeature {
    #[allow(dead_code)]
    uri: String,
    elements: Vec<CucumberElement>,
}

#[derive(Debug, Deserialize)]
struct CucumberElement {
    name: String,
    #[serde(rename = "type")]
    element_type: String,
    tags: Vec<CucumberTag>,
    steps: Vec<CucumberStep>,
}

#[derive(Debug, Deserialize)]
struct CucumberTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CucumberStep {
    result: CucumberStepResult,
}

#[derive(Debug, Deserialize)]
struct CucumberStepResult {
    status: String,
}

async fn get_coverage(State(state): State<AppState>) -> Json<CoverageResponse> {
    let root = &state.workspace_root;

    // Load spec ledger to get all ACs
    let specs = match load_all_specs(root) {
        Ok(s) => s,
        Err(_) => {
            // Return empty response if specs can't be loaded
            return Json(CoverageResponse {
                summary: CoverageSummary { passing: 0, failing: 0, unknown: 0, total: 0 },
                details: vec![],
            });
        }
    };

    // Build a map of all ACs from the ledger
    let mut ac_map: HashMap<String, (String, String, String)> = HashMap::new();
    for story in &specs.ledger.stories {
        for req in &story.requirements {
            for ac in &req.acceptance_criteria {
                ac_map.insert(ac.id.clone(), (story.id.clone(), req.id.clone(), ac.text.clone()));
            }
        }
    }

    // Try to parse BDD results from JSON report
    let bdd_json_path = root.join("target/ac_report.json");
    let mut ac_status: HashMap<String, String> = HashMap::new();
    let mut ac_scenarios: HashMap<String, Vec<String>> = HashMap::new();

    if bdd_json_path.exists()
        && let Ok(content) = fs::read_to_string(&bdd_json_path)
        && let Ok(report) = serde_json::from_str::<CucumberReport>(&content)
    {
        for feature in report.0 {
            for element in feature.elements {
                // Only process scenarios
                if element.element_type == "scenario" {
                    // Extract AC IDs from tags
                    let ac_ids: Vec<String> = element
                        .tags
                        .iter()
                        .filter_map(|tag| {
                            // Tags in Cucumber JSON include an @ prefix - normalize before matching
                            let tag_name = tag.name.trim_start_matches('@');
                            if tag_name.starts_with("AC-") {
                                Some(tag_name.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Determine if scenario passed (all steps passed)
                    let passed = element.steps.iter().all(|step| step.result.status == "passed");

                    // Update status and scenarios for each AC
                    for ac_id in ac_ids {
                        // Track scenario name
                        ac_scenarios.entry(ac_id.clone()).or_default().push(element.name.clone());

                        // Update status (if any scenario fails, AC fails)
                        let current_status = ac_status.entry(ac_id.clone()).or_insert_with(|| {
                            if passed { "passing".to_string() } else { "failing".to_string() }
                        });

                        if !passed {
                            *current_status = "failing".to_string();
                        }
                    }
                }
            }
        }
    }

    // Build details and compute summary
    let mut passing = 0;
    let mut failing = 0;
    let mut unknown = 0;
    let mut details = Vec::new();

    for (ac_id, (story_id, req_id, title)) in &ac_map {
        let status = ac_status.get(ac_id).cloned().unwrap_or_else(|| "unknown".to_string());
        let scenarios = ac_scenarios.get(ac_id).cloned().unwrap_or_default();

        match status.as_str() {
            "passing" => passing += 1,
            "failing" => failing += 1,
            _ => unknown += 1,
        }

        details.push(CoverageDetail {
            id: ac_id.clone(),
            title: title.clone(),
            status,
            story: story_id.clone(),
            requirement: req_id.clone(),
            scenarios,
        });
    }

    // Sort details by ID for consistent output
    details.sort_by(|a, b| a.id.cmp(&b.id));

    let total = passing + failing + unknown;

    Json(CoverageResponse {
        summary: CoverageSummary { passing, failing, unknown, total },
        details,
    })
}

async fn get_schema() -> Json<spec_runtime::PlatformSchemas> {
    Json(spec_runtime::get_all_schemas())
}

async fn get_schema_by_name_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<spec_runtime::SchemaInfo>, AppError> {
    spec_runtime::get_schema_by_name(&name)
        .map(Json)
        .ok_or_else(|| AppError::not_found(format!("Schema '{}' not found", name)))
}

fn task_status_to_string(status: TaskStatus) -> String {
    format!("{status:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{PlatformAuthConfig, PlatformAuthMode};

    #[test]
    fn log_hygiene_redacts_secrets() {
        let mut settings = HashMap::new();
        settings
            .insert("platform.auth_mode".to_string(), serde_yaml::Value::String("basic".into()));

        let mut secrets = HashMap::new();
        secrets.insert("db.url".to_string(), "postgres://user:pass@localhost:5432/app".to_string());
        secrets.insert("platform.auth_token".to_string(), "super-secret-token".to_string());

        let config =
            ValidatedConfig { http_port: 8080, env: Some("dev".to_string()), settings, secrets };

        let auth = PlatformAuthConfig {
            mode: PlatformAuthMode::Basic,
            token: Some("super-secret-token".into()),
        };
        let summary = ConfigSummary::from_parts(&config, &auth);

        let serialized = serde_json::to_string(&summary).expect("summary should serialize");

        assert!(
            !serialized.contains("super-secret-token"),
            "Serialized summary should not leak auth tokens"
        );
        assert_eq!(summary.secrets_redacted.get("db.url"), Some(&"[REDACTED]".to_string()));
        assert_eq!(summary.auth.mode, "basic");
        assert!(summary.auth.token_present);
    }

    #[test]
    fn normalizes_common_status_variants() {
        assert_eq!(normalize_status("open"), "Todo");
        assert_eq!(normalize_status("in_progress"), "InProgress");
        assert_eq!(normalize_status("in-progress"), "InProgress");
        assert_eq!(normalize_status("review"), "Review");
        assert_eq!(normalize_status("done"), "Done");
        assert_eq!(normalize_status("InProgress"), "InProgress");
    }

    #[test]
    fn defaults_unknown_statuses_to_todo() {
        assert_eq!(normalize_status("blocked"), "Todo");
        assert_eq!(normalize_status(""), "Todo");
    }
}

#[derive(Deserialize)]
struct TaskGraphQuery {
    format: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum TaskGraphResponse {
    Json(spec_runtime::tasks::TaskGraph),
    Mermaid { mermaid: String },
}

async fn get_task_graph(
    State(state): State<AppState>,
    Query(query): Query<TaskGraphQuery>,
) -> Result<Json<TaskGraphResponse>, AppError> {
    let root = &state.workspace_root;
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml")).map_err(|err| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            format!("Failed to load specs/tasks.yaml: {}", err),
        )
    })?;

    let graph = spec_runtime::tasks::build_task_graph(&tasks_spec);

    let response = match query.format.as_deref() {
        Some("mermaid") => {
            let mermaid = spec_runtime::tasks::generate_mermaid_diagram(&graph);
            TaskGraphResponse::Mermaid { mermaid }
        }
        _ => TaskGraphResponse::Json(graph),
    };

    Ok(Json(response))
}

```

# FILE: crates/app-http/src/platform/forks.rs

```
use crate::{AppError, AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Fork registry entry representing a known template fork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEntry {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub kernel_version: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<Maintainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pain_points: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub friction: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ForksListResponse {
    pub forks: Vec<ForkSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ForkSummary {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub status: String,
    pub kernel_version: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ForkRegistry {
    schema_version: String,
    template_version: String,
    #[serde(default)]
    forks: Vec<ForkRegistryEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ForkRegistryEntry {
    id: String,
    name: String,
    domain: String,
    status: String,
    kernel_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

/// Router for fork endpoints
pub fn router() -> Router<AppState> {
    Router::new().route("/forks", get(get_all_forks)).route("/forks/{name}", get(get_fork_by_name))
}

/// Load fork registry from fork_registry.yaml
#[allow(clippy::result_large_err)]
#[allow(dead_code)]
fn load_fork_registry(workspace_root: &std::path::Path) -> Result<ForkRegistry, AppError> {
    let registry_path = workspace_root.join("forks/fork_registry.yaml");

    if !registry_path.exists() {
        return Ok(ForkRegistry {
            schema_version: "1.0".to_string(),
            template_version: "v3.3.3".to_string(),
            forks: Vec::new(),
        });
    }

    let content = fs::read_to_string(&registry_path).map_err(|e| {
        AppError::internal_error(format!("Failed to read fork_registry.yaml: {}", e))
    })?;

    let registry: ForkRegistry = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse fork_registry.yaml: {}", e))
    })?;

    Ok(registry)
}

/// Load all fork entries from forks/ directory
#[allow(clippy::result_large_err)]
fn load_all_forks(workspace_root: &std::path::Path) -> Result<Vec<ForkEntry>, AppError> {
    let forks_dir = workspace_root.join("forks");

    if !forks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut forks = Vec::new();

    let dir_entries = fs::read_dir(&forks_dir)
        .map_err(|e| AppError::internal_error(format!("Failed to read forks directory: {}", e)))?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            AppError::internal_error(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and special files
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("README.yaml") | Some("fork_registry.yaml") | Some("fork_schema.yaml")
            )
        {
            continue;
        }

        // Only load files matching FORK-*.yaml pattern
        #[allow(clippy::collapsible_if)]
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if !filename.starts_with("FORK-") {
                continue;
            }
        }

        match load_fork_entry(&path) {
            Ok(fork) => forks.push(fork),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load fork entry"
                );
            }
        }
    }

    // Sort by ID for consistent output
    forks.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(forks)
}

/// Load a single fork entry from a YAML file
#[allow(clippy::result_large_err)]
fn load_fork_entry(path: &std::path::Path) -> Result<ForkEntry, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::internal_error(format!("Failed to read fork file {}: {}", path.display(), e))
    })?;

    let fork: ForkEntry = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse fork YAML {}: {}", path.display(), e))
    })?;

    Ok(fork)
}

/// GET /platform/forks - Get all fork entries
async fn get_all_forks(State(state): State<AppState>) -> Result<Json<ForksListResponse>, AppError> {
    let root = &state.workspace_root;
    let forks = load_all_forks(root)?;

    let summaries: Vec<ForkSummary> = forks
        .iter()
        .map(|f| ForkSummary {
            id: f.id.clone(),
            name: f.name.clone(),
            domain: f.domain.clone(),
            status: f.status.clone(),
            kernel_version: f.kernel_version.clone(),
        })
        .collect();

    let total = summaries.len();

    Ok(Json(ForksListResponse { forks: summaries, total }))
}

/// GET /platform/forks/:name - Get a specific fork entry by ID or name
async fn get_fork_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ForkEntry>, AppError> {
    let root = &state.workspace_root;
    let forks_dir = root.join("forks");

    // Try to find the fork file
    // It could be the full ID (FORK-XXX-001) or just the identifier
    let possible_filenames =
        vec![format!("{}.yaml", name), format!("FORK-{}.yaml", name.trim_start_matches("FORK-"))];

    for filename in possible_filenames {
        let file_path = forks_dir.join(&filename);
        if file_path.exists() {
            let fork = load_fork_entry(&file_path)?;

            // Verify the ID or name matches
            if fork.id != name && !fork.name.eq_ignore_ascii_case(&name) {
                tracing::warn!(
                    requested = %name,
                    found_id = %fork.id,
                    found_name = %fork.name,
                    file = %file_path.display(),
                    "Fork identifier mismatch"
                );
            }

            return Ok(Json(fork));
        }
    }

    Err(AppError::not_found(format!("Fork '{}' not found", name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_entry_serialization() {
        let fork = ForkEntry {
            id: "FORK-TEST-001".to_string(),
            name: "Test Fork".to_string(),
            domain: "testing".to_string(),
            kernel_version: "v3.3.3".to_string(),
            status: "active".to_string(),
            url: Some("https://github.com/test/fork".to_string()),
            maintainer: Some(Maintainer {
                name: "Test Maintainer".to_string(),
                contact: Some("test@example.com".to_string()),
            }),
            forked_at: Some("2025-11-26".to_string()),
            last_synced: None,
            features: vec!["feature1".to_string()],
            pain_points: vec![],
            notes: None,
            related_items: None,
        };

        let json = serde_json::to_string(&fork).unwrap();
        assert!(json.contains("FORK-TEST-001"));
        assert!(json.contains("testing"));
    }

    #[test]
    fn test_fork_entry_deserialization() {
        let yaml = r#"
id: FORK-TEST-002
name: "Test Fork 2"
domain: rust-sdk
kernel_version: v3.3.3
status: active
"#;

        let fork: ForkEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fork.id, "FORK-TEST-002");
        assert_eq!(fork.domain, "rust-sdk");
        assert_eq!(fork.status, "active");
    }

    #[test]
    fn test_fork_registry_deserialization() {
        let yaml = r#"
schema_version: "1.0"
template_version: "v3.3.3"
forks:
  - id: FORK-TEST-001
    name: "Test Fork"
    domain: testing
    status: active
    kernel_version: v3.3.3
    file: forks/FORK-TEST-001.yaml
"#;

        let registry: ForkRegistry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(registry.forks.len(), 1);
        assert_eq!(registry.forks[0].id, "FORK-TEST-001");
    }
}

```

# FILE: crates/app-http/src/platform/friction.rs

```
use crate::{AppError, AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Friction entry representing process/tooling issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionEntry {
    pub id: String,
    pub date: String,
    pub category: String,
    pub severity: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workaround: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FrictionContext>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<RelatedItems>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub resolved_by: String,
    pub resolved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pr_links: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FrictionListResponse {
    pub entries: Vec<FrictionEntry>,
    pub total: usize,
}

/// Router for friction endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/friction", get(get_all_friction))
        .route("/friction/{id}", get(get_friction_by_id))
}

/// Load all friction entries from friction/ directory
#[allow(clippy::result_large_err)]
fn load_all_friction_entries(
    workspace_root: &std::path::Path,
) -> Result<Vec<FrictionEntry>, AppError> {
    let friction_dir = workspace_root.join("friction");

    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let dir_entries = fs::read_dir(&friction_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to read friction directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            AppError::internal_error(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_friction_entry(&path) {
            Ok(friction) => entries.push(friction),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load friction entry"
                );
            }
        }
    }

    // Sort by date (most recent first)
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(entries)
}

/// Load a single friction entry from a YAML file
#[allow(clippy::result_large_err)]
fn load_friction_entry(path: &std::path::Path) -> Result<FrictionEntry, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::internal_error(format!("Failed to read friction file {}: {}", path.display(), e))
    })?;

    let entry: FrictionEntry = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse friction YAML {}: {}", path.display(), e))
    })?;

    Ok(entry)
}

/// GET /platform/friction - Get all friction entries
async fn get_all_friction(
    State(state): State<AppState>,
) -> Result<Json<FrictionListResponse>, AppError> {
    let root = &state.workspace_root;
    let entries = load_all_friction_entries(root)?;
    let total = entries.len();

    Ok(Json(FrictionListResponse { entries, total }))
}

/// GET /platform/friction/:id - Get a specific friction entry by ID
async fn get_friction_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FrictionEntry>, AppError> {
    let root = &state.workspace_root;
    let friction_dir = root.join("friction");

    // Construct the expected file path
    let file_path = friction_dir.join(format!("{}.yaml", id));

    // Check if file exists
    if !file_path.exists() {
        return Err(AppError::not_found(format!("Friction entry '{}' not found", id)));
    }

    // Load and return the entry
    let entry = load_friction_entry(&file_path)?;

    // Verify the ID matches (sanity check)
    if entry.id != id {
        return Err(AppError::internal_error(format!(
            "Friction entry ID mismatch: expected '{}', found '{}'",
            id, entry.id
        )));
    }

    Ok(Json(entry))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friction_entry_serialization() {
        let entry = FrictionEntry {
            id: "FRICTION-TEST-001".to_string(),
            date: "2025-11-26".to_string(),
            category: "testing".to_string(),
            severity: "low".to_string(),
            summary: "Test friction entry".to_string(),
            description: "Test description".to_string(),
            expected_behavior: None,
            workaround: None,
            impact: None,
            context: None,
            status: "open".to_string(),
            resolution: None,
            refs: Vec::new(),
            related_items: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("FRICTION-TEST-001"));
        assert!(json.contains("testing"));
    }

    #[test]
    fn test_friction_entry_deserialization() {
        let yaml = r#"
id: FRICTION-TEST-002
date: "2025-11-26"
category: devex
severity: medium
summary: "Test friction"
description: "Test description"
status: open
"#;

        let entry: FrictionEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.id, "FRICTION-TEST-002");
        assert_eq!(entry.category, "devex");
        assert_eq!(entry.status, "open");
    }

    #[test]
    fn test_default_status() {
        let yaml = r#"
id: FRICTION-TEST-003
date: "2025-11-26"
category: tooling
severity: high
summary: "Test"
description: "Test"
"#;

        let entry: FrictionEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.status, "open");
    }
}

```

# FILE: crates/app-http/src/platform/questions.rs

```
use crate::{AppError, AppState};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::fs;

/// Question artifact representing flow decision points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub req_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ac_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    pub summary: String,
    pub context: QuestionContext,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<QuestionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<Recommendation>,
    pub created_by: String,
    pub created_at: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<QuestionResolution>,
}

fn default_status() -> String {
    "open".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionContext {
    pub flow: String,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default = "default_reversible")]
    pub reversible: bool,
}

fn default_reversible() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub option_label: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResolution {
    pub resolved_by: String,
    pub resolved_at: String,
    pub chosen_option: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuestionsListResponse {
    pub questions: Vec<QuestionSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct QuestionSummary {
    pub id: String,
    pub summary: String,
    pub status: String,
    pub flow: String,
    pub phase: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct QuestionFilters {
    pub status: Option<String>,
}

/// Router for question endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/questions", get(get_all_questions))
        .route("/questions/{id}", get(get_question_by_id))
}

/// Load all question entries from questions/ directory
#[allow(clippy::result_large_err)]
fn load_all_questions(
    workspace_root: &std::path::Path,
    status_filter: Option<&str>,
) -> Result<Vec<Question>, AppError> {
    let questions_dir = workspace_root.join("questions");

    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    let dir_entries = fs::read_dir(&questions_dir).map_err(|e| {
        AppError::internal_error(format!("Failed to read questions directory: {}", e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            AppError::internal_error(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();

        // Skip non-YAML files and README
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
        {
            continue;
        }

        match load_question_entry(&path) {
            Ok(question) => {
                // Apply status filter if provided
                if let Some(filter_status) = status_filter {
                    if question.status.eq_ignore_ascii_case(filter_status) {
                        questions.push(question);
                    }
                } else {
                    questions.push(question);
                }
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load question entry"
                );
            }
        }
    }

    // Sort by created_at (most recent first)
    questions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(questions)
}

/// Load a single question entry from a YAML file
#[allow(clippy::result_large_err)]
fn load_question_entry(path: &std::path::Path) -> Result<Question, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::internal_error(format!("Failed to read question file {}: {}", path.display(), e))
    })?;

    let question: Question = serde_yaml::from_str(&content).map_err(|e| {
        AppError::internal_error(format!("Failed to parse question YAML {}: {}", path.display(), e))
    })?;

    Ok(question)
}

/// GET /platform/questions - Get all question entries (optionally filtered by status)
async fn get_all_questions(
    State(state): State<AppState>,
    Query(filters): Query<QuestionFilters>,
) -> Result<Json<QuestionsListResponse>, AppError> {
    let root = &state.workspace_root;
    let questions = load_all_questions(root, filters.status.as_deref())?;

    let summaries: Vec<QuestionSummary> = questions
        .iter()
        .map(|q| QuestionSummary {
            id: q.id.clone(),
            summary: q.summary.clone(),
            status: q.status.clone(),
            flow: q.context.flow.clone(),
            phase: q.context.phase.clone(),
            created_at: q.created_at.clone(),
        })
        .collect();

    let total = summaries.len();

    Ok(Json(QuestionsListResponse { questions: summaries, total }))
}

/// GET /platform/questions/:id - Get a specific question entry by ID
async fn get_question_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Question>, AppError> {
    let root = &state.workspace_root;
    let questions_dir = root.join("questions");

    // Try to find the question file
    // It could be named with or without a prefix (e.g., Q-EXAMPLE-001.yaml or QUESTION-001.yaml)
    let possible_filenames = vec![
        format!("{}.yaml", id),
        format!("Q-{}.yaml", id.trim_start_matches("Q-")),
        format!("QUESTION-{}.yaml", id.trim_start_matches("QUESTION-")),
    ];

    for filename in possible_filenames {
        let file_path = questions_dir.join(&filename);
        if file_path.exists() {
            let question = load_question_entry(&file_path)?;

            // Verify the ID matches (sanity check)
            if question.id != id {
                tracing::warn!(
                    expected_id = %id,
                    found_id = %question.id,
                    file = %file_path.display(),
                    "Question ID mismatch"
                );
            }

            return Ok(Json(question));
        }
    }

    Err(AppError::not_found(format!("Question '{}' not found", id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_serialization() {
        let question = Question {
            id: "Q-TEST-001".to_string(),
            task_id: Some("implement_feature".to_string()),
            req_ids: vec!["REQ-001".to_string()],
            ac_ids: vec!["AC-001".to_string()],
            refs: vec![],
            summary: "Test question".to_string(),
            context: QuestionContext {
                flow: "bundle".to_string(),
                phase: "selection".to_string(),
                description: Some("Test description".to_string()),
                files_involved: vec![],
            },
            options: vec![],
            recommendation: None,
            created_by: "flow".to_string(),
            created_at: "2025-11-26T00:00:00Z".to_string(),
            status: "open".to_string(),
            resolution: None,
        };

        let json = serde_json::to_string(&question).unwrap();
        assert!(json.contains("Q-TEST-001"));
        assert!(json.contains("bundle"));
    }

    #[test]
    fn test_question_deserialization() {
        let yaml = r#"
id: Q-TEST-002
task_id: implement_ac
req_ids:
  - REQ-001
ac_ids:
  - AC-001
summary: "Test question"
context:
  flow: bundle
  phase: selection
created_by: flow
created_at: "2025-11-26T00:00:00Z"
status: open
"#;

        let question: Question = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(question.id, "Q-TEST-002");
        assert_eq!(question.context.flow, "bundle");
        assert_eq!(question.status, "open");
    }

    #[test]
    fn test_default_status() {
        let yaml = r#"
id: Q-TEST-003
summary: "Test"
context:
  flow: test
  phase: test
created_by: flow
created_at: "2025-11-26T00:00:00Z"
"#;

        let question: Question = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(question.status, "open");
    }
}

```

# FILE: crates/app-http/src/platform/ui.rs

```
use axum::{extract::State, response::Html};
use maud::{DOCTYPE, Markup, html};
use spec_runtime::{ServiceMetadata, load_all_specs, load_service_metadata};

use super::config_summary;
use crate::AppState;

/// Shared layout for all UI pages
fn layout(title: &str, metadata: &Option<ServiceMetadata>, content: Markup) -> Markup {
    let service_name = metadata
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("Rust-as-Spec Platform");
    let service_tagline =
        metadata.as_ref().and_then(|m| m.description.as_deref()).unwrap_or_default();

    let links = metadata.as_ref().map(|m| m.links.clone()).unwrap_or_default();

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " - Rust-as-Spec Platform" }
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js" {}
                style {
                    r#"
                    * { margin: 0; padding: 0; box-sizing: border-box; }
                    body {
                        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        background: #f5f5f5;
                    }
                    .container {
                        max-width: 1200px;
                        margin: 0 auto;
                        padding: 20px;
                    }
                    header {
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        padding: 2rem;
                        margin-bottom: 2rem;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                    }
                    header h1 {
                        font-size: 2rem;
                        margin-bottom: 0.5rem;
                    }
                    header p {
                        opacity: 0.9;
                    }
                    nav {
                        background: white;
                        padding: 1rem;
                        margin-bottom: 2rem;
                        border-radius: 8px;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
                    }
                    nav a {
                        color: #667eea;
                        text-decoration: none;
                        margin-right: 2rem;
                        font-weight: 500;
                    }
                    nav a:hover {
                        text-decoration: underline;
                    }
                    .card {
                        background: white;
                        border-radius: 8px;
                        padding: 1.5rem;
                        margin-bottom: 1.5rem;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
                    }
                    .card h2 {
                        color: #667eea;
                        margin-bottom: 1rem;
                        font-size: 1.5rem;
                    }
                    .metrics {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                        gap: 1rem;
                    }
                    .metric {
                        padding: 1rem;
                        background: #f8f9fa;
                        border-radius: 6px;
                        border-left: 4px solid #667eea;
                    }
                    .metric-label {
                        font-size: 0.875rem;
                        color: #666;
                        margin-bottom: 0.25rem;
                    }
                    .metric-value {
                        font-size: 2rem;
                        font-weight: bold;
                        color: #333;
                    }
                    .status-badge {
                        display: inline-block;
                        padding: 0.25rem 0.75rem;
                        border-radius: 12px;
                        font-size: 0.875rem;
                        font-weight: 500;
                    }
                    .status-pass {
                        background: #d4edda;
                        color: #155724;
                    }
                    .status-fail {
                        background: #f8d7da;
                        color: #721c24;
                    }
                    .status-unknown {
                        background: #fff3cd;
                        color: #856404;
                    }
                    pre {
                        background: #f8f9fa;
                        padding: 1rem;
                        border-radius: 6px;
                        overflow-x: auto;
                    }
                    .mermaid {
                        background: white;
                        padding: 2rem;
                        border-radius: 8px;
                    }
                    "#
                }
                script {
                    "mermaid.initialize({ startOnLoad: true, theme: 'default' });"
                }
            }
            body {
                header {
                    .container {
                        h1 { (service_name) }
                        p { (service_tagline) }
                    }
                }
                nav .container {
                    a href="/" { "Dashboard" }
                    a href="/ui/graph" { "Graph" }
                    a href="/ui/flows" { "Flows & Tasks" }
                    a href="/ui/coverage" { "AC Coverage" }
                    a href="/platform/status" target="_blank" { "API: Status" }
                    a href="/platform/graph" target="_blank" { "API: Graph" }
                    @if let Some(runbook) = links.get("kernel_contract") {
                        a href=(runbook) target="_blank" { "Runbook" }
                    }
                    @if let Some(roadmap) = links.get("roadmap") {
                        a href=(roadmap) target="_blank" { "Roadmap" }
                    }
                    @if let Some(agent_guide) = links.get("agent_guide") {
                        a href=(agent_guide) target="_blank" { "Agent Guide" }
                    }
                    @if let Some(feature_status) = links.get("feature_status") {
                        a href=(feature_status) target="_blank" { "Feature Status" }
                    }
                    @if let Some(support) = links.get("support") {
                        a href=(support) target="_blank" { "Platform Support" }
                    }
                }
                main .container {
                    (content)
                }
            }
        }
    }
}

/// Dashboard page
pub async fn dashboard(State(state): State<AppState>) -> Html<String> {
    let root = &state.workspace_root;
    let status_result = load_all_specs(root);
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();
    let config = config_summary(&state);

    let content = match (status_result, tasks_result) {
        (Ok(specs), Ok(tasks_spec)) => {
            let req_count: usize = specs.ledger.stories.iter().map(|s| s.requirements.len()).sum();
            let ac_count: usize = specs
                .ledger
                .stories
                .iter()
                .flat_map(|s| s.requirements.iter())
                .map(|r| r.acceptance_criteria.len())
                .sum();

            // Read policy status
            let policy_path = root.join("target/policy_status.json");
            let policy_status = std::fs::read_to_string(policy_path)
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|v| v.get("summary").and_then(|s| s.as_str()).map(String::from))
                .unwrap_or_else(|| "unknown".to_string());

            let status_class = match policy_status.as_str() {
                "pass" => "status-pass",
                "fail" => "status-fail",
                _ => "status-unknown",
            };

            // Read AC coverage from feature_status.md
            let feature_status_path = root.join("docs/feature_status.md");
            let mut passing = 0;
            let mut failing = 0;
            let mut unknown = 0;
            let mut coverage_rows = 0;

            if feature_status_path.exists()
                && let Ok(content) = std::fs::read_to_string(&feature_status_path)
            {
                for line in content.lines() {
                    if !line.starts_with("| AC-") {
                        continue;
                    }

                    coverage_rows += 1;

                    if line.contains("[PASS]") {
                        passing += 1;
                    } else if line.contains("[FAIL]") {
                        failing += 1;
                    } else if line.contains("[UNKNOWN]") {
                        unknown += 1;
                    }
                }
            }

            // If no coverage data, count all ACs as unknown
            if coverage_rows == 0 {
                unknown = ac_count;
            } else {
                let accounted_for = passing + failing + unknown;
                if accounted_for < ac_count {
                    unknown += ac_count - accounted_for;
                }
            }

            html! {
                .card {
                    h2 { "Platform Health" }
                    .metrics {
                        .metric {
                            .metric-label { "Stories" }
                            .metric-value { (specs.ledger.stories.len()) }
                        }
                        .metric {
                            .metric-label { "Requirements" }
                            .metric-value { (req_count) }
                        }
                        .metric {
                            .metric-label { "Acceptance Criteria" }
                            .metric-value { (ac_count) }
                        }
                        .metric {
                            .metric-label { "DevEx Commands" }
                            .metric-value { (specs.devex.commands.len()) }
                        }
                        .metric {
                            .metric-label { "Flows" }
                            .metric-value { (specs.devex.flows.len()) }
                        }
                        .metric {
                            .metric-label { "Tasks" }
                            .metric-value { (tasks_spec.tasks.len()) }
                        }
                        .metric {
                            .metric-label { "Documents" }
                            .metric-value { (specs.docs.docs.len()) }
                        }
                        .metric {
                            .metric-label { "Policies" }
                            .metric-value {
                                span class=(status_class) { (policy_status) }
                            }
                        }
                    }
                }

                .card {
                    h2 { "AC Coverage" }
                    .stats style="display: flex; gap: 1.5rem; margin: 1rem 0; flex-wrap: wrap;" {
                        span style="color: #155724; font-size: 1.1rem; font-weight: 500;" {
                            "✅ " (passing) " passing"
                        }
                        span style="color: #721c24; font-size: 1.1rem; font-weight: 500;" {
                            "❌ " (failing) " failing"
                        }
                        span style="color: #856404; font-size: 1.1rem; font-weight: 500;" {
                            "❓ " (unknown) " unknown"
                        }
                    }
                    p style="margin-top: 1rem;" {
                        a href="/ui/coverage" style="color: #667eea; text-decoration: none; font-weight: 500;" {
                            "View details →"
                        }
                    }
                }

                @if let Some(cfg) = config.clone() {
                    .card {
                        h2 { "Runtime Config (redacted)" }
                        p style="margin-bottom: 0.75rem; color: #555;" {
                            "Config values are rendered for visibility without leaking secrets; tokens are never shown."
                        }
                        ul style="margin-left: 1.25rem; line-height: 1.6;" {
                            li {
                                strong { "Env: " }
                                (cfg.env.clone().unwrap_or_else(|| "unknown".to_string()))
                            }
                            li { strong { "HTTP port: " } (cfg.http_port) }
                            li {
                                strong { "Auth mode: " }
                                (cfg.auth.mode.clone())
                                " ("
                                @if cfg.auth.token_present { "token configured" } @else { "no token" }
                                ")"
                            }
                        }

                        @if !cfg.settings.is_empty() {
                            details style="margin-top: 0.75rem;" {
                                summary style="cursor: pointer; color: #667eea;" { "Settings" }
                                ul style="margin: 0.5rem 0 0 1.25rem;" {
                                    @for (k, v) in cfg.settings.iter() {
                                        li { code { (k) } ": " (v) }
                                    }
                                }
                            }
                        }

                        @if !cfg.secrets_redacted.is_empty() {
                            details style="margin-top: 0.75rem;" {
                                summary style="cursor: pointer; color: #667eea;" { "Secrets (redacted)" }
                                ul style="margin: 0.5rem 0 0 1.25rem;" {
                                    @for (k, _) in cfg.secrets_redacted.iter() {
                                        li { code { (k) } ": " "[REDACTED]" }
                                    }
                                }
                            }
                        }
                    }
                }

                .card {
                    h2 { "Governance Contracts" }
                    p { "All governance checks are enforced via " code { "cargo xtask selftest" } ":" }
                    ul style="margin: 1rem 0 0 2rem;" {
                        li { "✅ Core checks (fmt, clippy, tests)" }
                        li { "✅ BDD acceptance tests" }
                        li { "✅ AC status mapping & ADR references" }
                        li { "✅ LLM context bundler" }
                        li { "✅ Policy tests " span class=(status_class) { "(" (policy_status) ")" } }
                        li { "✅ DevEx contract satisfaction" }
                        li { "✅ Graph invariants" }
                    }
                }

                .card {
                    h2 { "Quick Links" }
                    ul style="margin: 1rem 0 0 2rem;" {
                        li { a href="/ui/graph" { "View Governance Graph" } " - Visual map of stories, requirements, and ACs" }
                        li { a href="/ui/flows" { "View Flows & Tasks" } " - Developer workflows and task guidance" }
                        li { a href="/platform/status" target="_blank" { "Platform Status API" } " - JSON metrics for agents" }
                        li { a href="/platform/graph" target="_blank" { "Graph API" } " - Full governance graph as JSON" }
                    }
                }
            }
        }
        _ => {
            html! {
                .card {
                    h2 { "Error" }
                    p { "Failed to load platform specifications. Ensure specs are valid and available." }
                }
            }
        }
    };

    Html(layout("Dashboard", &metadata, content).into_string())
}

/// Graph visualization page
pub async fn graph_view(State(state): State<AppState>) -> Html<String> {
    let root = &state.workspace_root;
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

    let content = match load_all_specs(root) {
        Ok(specs) => match spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs) {
            Ok(graph) => {
                let mermaid_diagram = graph.to_mermaid();

                html! {
                    .card {
                        h2 { "Governance Graph" }
                        p style="margin-bottom: 1rem;" {
                            "This graph shows the relationships between stories, requirements, acceptance criteria, "
                            "documentation, DevEx commands, and flows."
                        }
                        .mermaid {
                            (mermaid_diagram)
                        }
                    }
                }
            }
            Err(e) => {
                html! {
                    .card {
                        h2 { "Error Building Graph" }
                        pre { (format!("{:?}", e)) }
                    }
                }
            }
        },
        Err(e) => {
            html! {
                .card {
                    h2 { "Error Loading Specs" }
                    pre { (format!("{:?}", e)) }
                }
            }
        }
    };

    Html(layout("Graph", &metadata, content).into_string())
}

/// Flows and tasks page
pub async fn flows_view(State(state): State<AppState>) -> Html<String> {
    let root = &state.workspace_root;
    let metadata = load_service_metadata(&root.join("specs/service_metadata.yaml")).ok();

    let flows_result = spec_runtime::load_devex_flows(&root.join("specs/devex_flows.yaml"));
    let tasks_result = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"));

    let content = match (flows_result, tasks_result) {
        (Ok(devex), Ok(tasks_spec)) => {
            html! {
                .card {
                    h2 { "DevEx Flows" }
                    p style="margin-bottom: 1rem;" {
                        "Developer experience flows define common workflows for working with this repository."
                    }
                    @for flow in devex.flows.values() {
                        .metric style="margin-bottom: 1rem;" {
                            h3 style="color: #667eea; font-size: 1.1rem;" { (flow.name) }
                            p style="color: #666; margin: 0.5rem 0;" { (flow.description) }
                            details {
                                summary style="cursor: pointer; color: #667eea;" { "Steps (" (flow.steps.len()) ")" }
                                ol style="margin: 0.5rem 0 0 2rem;" {
                                    @for step in &flow.steps {
                                        li { code { "cargo xtask " (step) } }
                                    }
                                }
                            }
                        }
                    }
                }

                .card {
                    h2 { "Tasks" }
                    p style="margin-bottom: 1rem;" {
                        "Tasks represent concrete work items with recommended flows and suggested sequences."
                    }
                    @for task in &tasks_spec.tasks {
                        .metric style="margin-bottom: 1rem;" {
                            h3 style="color: #667eea; font-size: 1.1rem;" { (task.title) }
                            p style="color: #666; margin: 0.5rem 0;" { (task.summary) }
                            p style="font-size: 0.875rem; margin: 0.5rem 0;" {
                                strong { "Status: " } (task.status)
                                " | "
                                strong { "Requirement: " } (task.requirement)
                            }
                            details {
                                summary style="cursor: pointer; color: #667eea;" {
                                    "View suggested sequence"
                                }
                                p style="margin: 0.5rem 0; font-size: 0.875rem;" {
                                    "Run: " code { "cargo xtask suggest-next --task " (task.id) }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            html! {
                .card {
                    h2 { "Error" }
                    p { "Failed to load flows or tasks." }
                }
            }
        }
    };

    Html(layout("Flows & Tasks", &metadata, content).into_string())
}

/// Coverage details page
pub async fn coverage_view(State(state): State<AppState>) -> Html<String> {
    let metadata =
        load_service_metadata(&state.workspace_root.join("specs/service_metadata.yaml")).ok();
    let content = html! {
        style {
            r#"
            .filter-controls {
                margin-bottom: 1.5rem;
                display: flex;
                gap: 1rem;
                align-items: center;
                flex-wrap: wrap;
            }
            .filter-btn {
                padding: 0.5rem 1rem;
                border: 2px solid #667eea;
                background: white;
                color: #667eea;
                border-radius: 6px;
                cursor: pointer;
                font-weight: 500;
                transition: all 0.2s;
            }
            .filter-btn:hover {
                background: #667eea;
                color: white;
            }
            .filter-btn.active {
                background: #667eea;
                color: white;
            }
            .search-box {
                flex: 1;
                min-width: 250px;
                padding: 0.5rem 1rem;
                border: 2px solid #ddd;
                border-radius: 6px;
                font-size: 1rem;
            }
            .search-box:focus {
                outline: none;
                border-color: #667eea;
            }
            .coverage-table {
                width: 100%;
                border-collapse: collapse;
                background: white;
            }
            .coverage-table th {
                background: #f8f9fa;
                padding: 0.75rem;
                text-align: left;
                font-weight: 600;
                border-bottom: 2px solid #dee2e6;
                position: sticky;
                top: 0;
            }
            .coverage-table td {
                padding: 0.75rem;
                border-bottom: 1px solid #dee2e6;
                vertical-align: top;
            }
            .coverage-table tr:hover {
                background: #f8f9fa;
            }
            .ac-row {
                transition: opacity 0.2s;
            }
            .ac-row.hidden {
                display: none;
            }
            .scenario-list {
                margin: 0;
                padding-left: 1.5rem;
                font-size: 0.875rem;
            }
            .scenario-list li {
                margin: 0.25rem 0;
            }
            "#
        }
        script {
            r#"
            let currentFilter = 'all';
            let allData = [];

            // Fetch coverage data on page load
            fetch('/platform/coverage')
                .then(res => res.json())
                .then(data => {
                    allData = data.details;
                    updateSummary(data.summary);
                    renderTable(allData);
                })
                .catch(err => {
                    console.error('Failed to load coverage data:', err);
                    document.getElementById('table-container').innerHTML =
                        '<p style="color: red;">Failed to load coverage data. Please try again.</p>';
                });

            function updateSummary(summary) {
                document.getElementById('passing-count').textContent = summary.passing;
                document.getElementById('failing-count').textContent = summary.failing;
                document.getElementById('unknown-count').textContent = summary.unknown;
                document.getElementById('total-count').textContent = summary.total;
            }

            function filterData(status) {
                currentFilter = status;

                // Update active button
                document.querySelectorAll('.filter-btn').forEach(btn => {
                    btn.classList.remove('active');
                });
                document.getElementById('filter-' + status).classList.add('active');

                // Apply filter
                applyFilters();
            }

            function searchData() {
                applyFilters();
            }

            function applyFilters() {
                const searchTerm = document.getElementById('search-box').value.toLowerCase();
                const rows = document.querySelectorAll('.ac-row');

                rows.forEach(row => {
                    const status = row.dataset.status;
                    const text = row.textContent.toLowerCase();

                    const statusMatch = currentFilter === 'all' || status === currentFilter;
                    const searchMatch = searchTerm === '' || text.includes(searchTerm);

                    if (statusMatch && searchMatch) {
                        row.classList.remove('hidden');
                    } else {
                        row.classList.add('hidden');
                    }
                });
            }

            function renderTable(data) {
                const tbody = document.getElementById('coverage-tbody');
                tbody.innerHTML = '';

                data.forEach(ac => {
                    const row = document.createElement('tr');
                    row.className = 'ac-row';
                    row.dataset.status = ac.status;

                    const statusBadge = ac.status === 'passing' ? '✅ pass' :
                                       ac.status === 'failing' ? '❌ fail' :
                                       '❓ unknown';
                    const badgeClass = ac.status === 'passing' ? 'status-pass' :
                                      ac.status === 'failing' ? 'status-fail' :
                                      'status-unknown';

                    const scenarios = ac.scenarios.length > 0
                        ? '<ul class="scenario-list">' +
                          ac.scenarios.map(s => '<li>' + s + '</li>').join('') +
                          '</ul>'
                        : '<em style="color: #999;">No scenarios</em>';

                    row.innerHTML = `
                        <td><code>${ac.id}</code></td>
                        <td>${ac.title}</td>
                        <td><span class="status-badge ${badgeClass}">${statusBadge}</span></td>
                        <td><code>${ac.story}</code></td>
                        <td><code>${ac.requirement}</code></td>
                        <td>${scenarios}</td>
                    `;

                    tbody.appendChild(row);
                });
            }

            // Initialize with 'all' filter active
            window.addEventListener('DOMContentLoaded', () => {
                document.getElementById('filter-all').classList.add('active');
            });
            "#
        }

        .card {
            h2 { "AC Coverage Summary" }
            .metrics {
                .metric style="border-left-color: #155724;" {
                    .metric-label { "Passing" }
                    .metric-value style="color: #155724;" id="passing-count" { "..." }
                }
                .metric style="border-left-color: #721c24;" {
                    .metric-label { "Failing" }
                    .metric-value style="color: #721c24;" id="failing-count" { "..." }
                }
                .metric style="border-left-color: #856404;" {
                    .metric-label { "Unknown" }
                    .metric-value style="color: #856404;" id="unknown-count" { "..." }
                }
                .metric {
                    .metric-label { "Total" }
                    .metric-value id="total-count" { "..." }
                }
            }
        }

        .card {
            h2 { "Acceptance Criteria Coverage" }
            .filter-controls {
                button #filter-all.filter-btn onclick="filterData('all')" { "All" }
                button #filter-passing.filter-btn onclick="filterData('passing')" { "Passing" }
                button #filter-failing.filter-btn onclick="filterData('failing')" { "Failing" }
                button #filter-unknown.filter-btn onclick="filterData('unknown')" { "Unknown" }
                input #search-box.search-box type="text" placeholder="Search by AC ID or title..."
                    oninput="searchData()";
            }

            #table-container {
                table .coverage-table {
                    thead {
                        tr {
                            th { "AC ID" }
                            th { "Title" }
                            th { "Status" }
                            th { "Story" }
                            th { "Requirement" }
                            th { "Scenarios" }
                        }
                    }
                    tbody #coverage-tbody {
                        tr {
                            td colspan="6" style="text-align: center; padding: 2rem; color: #999;" {
                                "Loading coverage data..."
                            }
                        }
                    }
                }
            }
        }
    };

    Html(layout("AC Coverage", &metadata, content).into_string())
}

```

# FILE: crates/app-http/src/security.rs

```
use spec_runtime::ValidatedConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformAuthMode {
    Open,
    Basic,
}

#[derive(Clone, Debug)]
pub struct PlatformAuthConfig {
    pub mode: PlatformAuthMode,
    pub token: Option<String>,
}

impl PlatformAuthConfig {
    pub fn from_sources(config: Option<&ValidatedConfig>) -> Self {
        let mode_raw = std::env::var("PLATFORM_AUTH_MODE")
            .ok()
            .or_else(|| {
                config
                    .and_then(|cfg| cfg.settings.get("platform.auth_mode"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "open".to_string());

        let token = std::env::var("PLATFORM_AUTH_TOKEN")
            .ok()
            .or_else(|| config.and_then(|cfg| cfg.secrets.get("platform.auth_token").cloned()));

        Self { mode: PlatformAuthMode::from(mode_raw.as_str()), token }
    }

    pub fn requires_auth(&self) -> bool {
        matches!(self.mode, PlatformAuthMode::Basic)
    }

    pub fn is_authorized(&self, provided: Option<&str>) -> bool {
        if !self.requires_auth() {
            return true;
        }

        match (self.token.as_deref(), provided) {
            (Some(expected), Some(candidate)) => constant_time_eq(expected, candidate),
            _ => false,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            PlatformAuthMode::Open => "open",
            PlatformAuthMode::Basic => "basic",
        }
    }

    pub fn token_present(&self) -> bool {
        self.token.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    /// Emit a warning when `basic` auth is enabled without a token.
    ///
    /// Returns `true` when a warning was emitted so tests can assert the behavior without
    /// scraping logs.
    pub fn warn_if_misconfigured(&self) -> bool {
        let misconfigured = self.requires_auth() && !self.token_present();
        if misconfigured {
            tracing::warn!(
                "Platform auth is set to basic but no token was provided; writes will be rejected"
            );
        }
        misconfigured
    }
}

impl From<&str> for PlatformAuthMode {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "basic" => PlatformAuthMode::Basic,
            "none" => PlatformAuthMode::Open,
            _ => PlatformAuthMode::Open,
        }
    }
}

// Simple constant-time comparison to avoid leaking length/case differences in tokens.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auth_mode_and_warns_on_missing_token() {
        let config = PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: None };
        assert!(config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(!config.is_authorized(Some("anything")));
    }

    #[test]
    fn accepts_correct_token_in_basic_mode() {
        let config =
            PlatformAuthConfig { mode: PlatformAuthMode::Basic, token: Some("secret".into()) };

        assert!(!config.warn_if_misconfigured());
        assert!(config.requires_auth());
        assert!(config.is_authorized(Some("secret")));
        assert!(!config.is_authorized(Some("other")));
    }

    #[test]
    fn open_mode_requires_no_token() {
        let config = PlatformAuthConfig { mode: PlatformAuthMode::Open, token: None };
        assert!(!config.requires_auth());
        assert!(config.is_authorized(None));
        assert!(config.is_authorized(Some("anything")));
    }
}

```

# FILE: crates/app-http/src/tasks.rs

```
use crate::{AppError, ErrorCode};
use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use business_core::governance::{TaskId, TaskService, TaskStatus};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateTaskStatusRequest {
    status: TaskStatus,
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let payload = parse_update_task_status(&headers, &body)?;
    let service = TaskService::new(state.governance_repo.clone());
    service.move_task(&TaskId(id), payload.status)?;
    Ok(StatusCode::NO_CONTENT)
}

#[allow(clippy::result_large_err)] // AppError is shared across handlers; keep signature consistent
fn parse_update_task_status(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<UpdateTaskStatusRequest, AppError> {
    let content_type =
        headers.get(axum::http::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("");

    if content_type.starts_with("application/json") {
        return serde_json::from_slice(body).map_err(|err| {
            AppError::validation_error(ErrorCode::InvalidRequest, format!("Invalid JSON: {}", err))
        });
    }

    if content_type.starts_with("application/x-www-form-urlencoded") {
        return serde_urlencoded::from_bytes(body).map_err(|err| {
            AppError::validation_error(
                ErrorCode::InvalidRequest,
                format!("Invalid form data: {}", err),
            )
        });
    }

    // Fallback: try to parse as JSON first, then form data to be forgiving
    if let Ok(value) = serde_json::from_slice(body) {
        return Ok(value);
    }

    if let Ok(value) = serde_urlencoded::from_bytes(body) {
        return Ok(value);
    }

    Err(AppError::validation_error(
        ErrorCode::InvalidRequest,
        "Unsupported body format; use JSON or x-www-form-urlencoded",
    ))
}

pub async fn tasks_ui(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let service = TaskService::new(state.governance_repo.clone());
    let tasks = service.list_tasks()?;

    let mut todo = Vec::new();
    let mut in_progress = Vec::new();
    let mut review = Vec::new();
    let mut done = Vec::new();

    for task in tasks {
        match task.status {
            TaskStatus::Todo => todo.push(task),
            TaskStatus::InProgress => in_progress.push(task),
            TaskStatus::Review => review.push(task),
            TaskStatus::Done => done.push(task),
        }
    }

    let render_column = |title: &str, tasks: Vec<business_core::governance::Task>| -> String {
        let tasks_html = tasks.into_iter().map(|t| {
            let buttons = match t.status {
                TaskStatus::Todo => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "InProgress"}}' hx-target="body">Start</button>"#, t.id.0),
                TaskStatus::InProgress => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Review"}}' hx-target="body">Review</button>"#, t.id.0),
                TaskStatus::Review => format!(r#"<button hx-post="/platform/tasks/{}/status" hx-vals='{{"status": "Done"}}' hx-target="body">Done</button>"#, t.id.0),
                TaskStatus::Done => String::new(),
            };

            format!(
                r#"<div class="task-card">
                    <h3>{}</h3>
                    <p>{}</p>
                    <div class="actions">{}</div>
                </div>"#,
                t.id.0, t.title, buttons
            )
        }).collect::<Vec<_>>().join("\n");

        format!(
            r#"<div class="column">
                <h2>{}</h2>
                <div class="task-list">
                    {}
                </div>
            </div>"#,
            title, tasks_html
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Task Board</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <style>
        body {{ font-family: sans-serif; padding: 20px; }}
        .board {{ display: flex; gap: 20px; }}
        .column {{ flex: 1; background: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .task-card {{ background: white; padding: 10px; margin-bottom: 10px; border-radius: 3px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .actions {{ margin-top: 10px; }}
        button {{ cursor: pointer; padding: 5px 10px; }}
    </style>
</head>
<body>
    <h1>Task Board</h1>
    <div class="board">
        {}
        {}
        {}
        {}
    </div>
</body>
</html>"#,
        render_column("Todo", todo),
        render_column("In Progress", in_progress),
        render_column("Review", review),
        render_column("Done", done)
    );

    Ok(Html(html))
}

```

# FILE: crates/app-http/tests/forks_api.rs

```
use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper to resolve workspace root from test binary location
fn test_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[tokio::test]
async fn test_get_all_forks() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("forks").is_some(), "Response should have 'forks' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let forks = json["forks"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(forks.len() as u64, total, "Total count should match forks length");

    // Verify fork entries have expected fields
    if let Some(first_fork) = forks.first() {
        assert!(first_fork.get("id").is_some());
        assert!(first_fork.get("name").is_some());
        assert!(first_fork.get("domain").is_some());
        assert!(first_fork.get("status").is_some());
        assert!(first_fork.get("kernel_version").is_some());
    }
}

#[tokio::test]
async fn test_get_fork_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/forks/FORK-NONEXISTENT-999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify error response structure
    assert!(json.get("error").is_some(), "Error response should have 'error' field");
    assert!(json.get("message").is_some(), "Error response should have 'message' field");

    let message = json["message"].as_str().unwrap();
    assert!(message.contains("FORK-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_forks_response_is_valid_json() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

    // Verify we can parse the response as JSON
    let _json: Value = serde_json::from_slice(&body).expect("Response should be valid JSON");
}

#[tokio::test]
async fn test_forks_sorted_by_id() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/forks").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let forks = json["forks"].as_array().unwrap();

    // Verify forks are sorted by ID
    if forks.len() > 1 {
        let ids: Vec<&str> = forks.iter().filter_map(|f| f["id"].as_str()).collect();

        for i in 0..ids.len() - 1 {
            assert!(
                ids[i] <= ids[i + 1],
                "Forks should be sorted by ID ascending (got {} before {})",
                ids[i],
                ids[i + 1]
            );
        }
    }
}

```

# FILE: crates/app-http/tests/friction_api.rs

```
use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper to resolve workspace root from test binary location
fn test_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[tokio::test]
async fn test_get_all_friction_entries() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/friction").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("entries").is_some(), "Response should have 'entries' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let entries = json["entries"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(entries.len() as u64, total, "Total count should match entries length");

    // Verify we have at least the test friction entries
    assert!(total >= 2, "Should have at least 2 friction entries");

    // Verify entries have expected fields
    if let Some(first_entry) = entries.first() {
        assert!(first_entry.get("id").is_some());
        assert!(first_entry.get("date").is_some());
        assert!(first_entry.get("category").is_some());
        assert!(first_entry.get("severity").is_some());
        assert!(first_entry.get("summary").is_some());
        assert!(first_entry.get("description").is_some());
        assert!(first_entry.get("status").is_some());
    }
}

#[tokio::test]
async fn test_get_friction_by_id_success() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/friction/FRICTION-AGENT-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the specific friction entry
    assert_eq!(json["id"].as_str().unwrap(), "FRICTION-AGENT-001");
    assert_eq!(json["category"].as_str().unwrap(), "api");
    assert_eq!(json["severity"].as_str().unwrap(), "high");
    assert_eq!(json["status"].as_str().unwrap(), "resolved");
    assert!(json["summary"].as_str().unwrap().contains("UI/API inconsistency"));
}

#[tokio::test]
async fn test_get_friction_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/friction/FRICTION-NONEXISTENT-999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify error response structure
    assert!(json.get("error").is_some(), "Error response should have 'error' field");
    assert!(json.get("message").is_some(), "Error response should have 'message' field");
    assert!(json.get("requestId").is_some(), "Error response should have 'requestId' field");

    let message = json["message"].as_str().unwrap();
    assert!(message.contains("FRICTION-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_friction_entries_sorted_by_date() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/friction").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let entries = json["entries"].as_array().unwrap();

    // Verify entries are sorted by date (most recent first)
    if entries.len() > 1 {
        let dates: Vec<&str> = entries.iter().filter_map(|e| e["date"].as_str()).collect();

        for i in 0..dates.len() - 1 {
            assert!(
                dates[i] >= dates[i + 1],
                "Entries should be sorted by date descending (most recent first)"
            );
        }
    }
}

```

# FILE: crates/app-http/tests/questions_api.rs

```
use adapters_spec_fs::FsGovernanceRepository;
use app_http::app_with_workspace_root;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper to resolve workspace root from test binary location
fn test_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[tokio::test]
async fn test_get_all_questions() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/questions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.get("questions").is_some(), "Response should have 'questions' field");
    assert!(json.get("total").is_some(), "Response should have 'total' field");

    let questions = json["questions"].as_array().unwrap();
    let total = json["total"].as_u64().unwrap();

    assert_eq!(questions.len() as u64, total, "Total count should match questions length");

    // Verify we have at least the example question
    assert!(total >= 1, "Should have at least 1 question entry");

    // Verify questions have expected fields
    if let Some(first_question) = questions.first() {
        assert!(first_question.get("id").is_some());
        assert!(first_question.get("summary").is_some());
        assert!(first_question.get("status").is_some());
        assert!(first_question.get("flow").is_some());
        assert!(first_question.get("phase").is_some());
        assert!(first_question.get("created_at").is_some());
    }
}

#[tokio::test]
async fn test_get_questions_filtered_by_status() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder().uri("/platform/questions?status=open").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let questions = json["questions"].as_array().unwrap();

    // All returned questions should have status "open"
    for question in questions {
        assert_eq!(question["status"].as_str().unwrap(), "open");
    }
}

#[tokio::test]
async fn test_get_question_by_id_success() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/questions/Q-EXAMPLE-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify the specific question entry
    assert_eq!(json["id"].as_str().unwrap(), "Q-EXAMPLE-001");
    assert_eq!(json["context"]["flow"].as_str().unwrap(), "bundle");
    assert_eq!(json["context"]["phase"].as_str().unwrap(), "ac_selection");
    assert!(json["summary"].as_str().unwrap().contains("multiple ACs"));
    assert!(json.get("options").is_some());
    assert!(json.get("recommendation").is_some());
}

#[tokio::test]
async fn test_get_question_by_id_not_found() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/platform/questions/Q-NONEXISTENT-999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify error response structure
    assert!(json.get("error").is_some(), "Error response should have 'error' field");
    assert!(json.get("message").is_some(), "Error response should have 'message' field");

    let message = json["message"].as_str().unwrap();
    assert!(message.contains("Q-NONEXISTENT-999"));
    assert!(message.contains("not found"));
}

#[tokio::test]
async fn test_questions_sorted_by_created_at() {
    let workspace_root = test_workspace_root();
    let repo = Arc::new(FsGovernanceRepository::new(workspace_root.clone()));
    let app = app_with_workspace_root(repo, workspace_root.clone());

    let response = app
        .oneshot(Request::builder().uri("/platform/questions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let questions = json["questions"].as_array().unwrap();

    // Verify questions are sorted by created_at (most recent first)
    if questions.len() > 1 {
        let dates: Vec<&str> = questions.iter().filter_map(|q| q["created_at"].as_str()).collect();

        for i in 0..dates.len() - 1 {
            assert!(
                dates[i] >= dates[i + 1],
                "Questions should be sorted by created_at descending (most recent first)"
            );
        }
    }
}

```

# FILE: crates/app-http/tests/task_status_api.rs

```
use adapters_spec_fs::tasks_state;
use app_http::{app_with_workspace_root, platform::TasksResponse};
use axum::{body::Body, http::Request};
use business_core::governance::{TaskId, TaskStatus};
use http_body_util::BodyExt;
use spec_runtime::tasks::{Task, TaskDocs, TasksSpec};
use std::{fs, sync::Arc};
use tempfile::tempdir;
use tower::ServiceExt;

fn write_tasks_files(spec_root: &std::path::Path, task_id: &str, status: TaskStatus) {
    let specs_dir = spec_root.join("specs");
    fs::create_dir_all(&specs_dir).expect("failed to create specs dir");

    // Seed tasks_state.yaml with the desired status
    let state_path = specs_dir.join("tasks_state.yaml");
    tasks_state::update_task_status(&state_path, TaskId(task_id.to_string()), status)
        .expect("failed to write tasks_state.yaml");

    // Seed tasks.yaml with a matching task definition
    let tasks_yaml = specs_dir.join("tasks.yaml");
    let tasks = TasksSpec {
        schema_version: "1.0.0".to_string(),
        template_version: "0.1.0".to_string(),
        tasks: vec![Task {
            id: task_id.to_string(),
            title: "Test Task".to_string(),
            requirement: "REQ-TPL-TEST".to_string(),
            acs: vec![],
            status: "Todo".to_string(),
            owner: None,
            labels: vec![],
            docs: Some(TaskDocs { design: vec![], plan: vec![] }),
            summary: "Test task summary".to_string(),
            recommended_flows: vec![],
            depends_on: vec![],
        }],
    };

    let content = serde_yaml::to_string(&tasks).expect("failed to serialize tasks.yaml");
    fs::write(tasks_yaml, content).expect("failed to write tasks.yaml");
}

#[tokio::test]
async fn update_task_status_endpoint_accepts_json_body() {
    // Use an isolated temp repo
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-001", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone());

    let body = r#"{ "status": "InProgress" }"#;
    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-001/status")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    if !status.is_success() {
        let body =
            BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    // Verify the status was persisted
    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-001".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}

#[tokio::test]
async fn update_task_status_endpoint_accepts_form_body() {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-002", TaskStatus::Todo);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone());

    let request = Request::builder()
        .method("POST")
        .uri("/platform/tasks/TASK-002/status")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("status=InProgress"))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    if !status.is_success() {
        let body =
            BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let state_path = spec_root.join("specs/tasks_state.yaml");
    let stored_status =
        tasks_state::get_task_status(&state_path, &TaskId("TASK-002".to_string())).unwrap();
    assert_eq!(stored_status, Some(TaskStatus::InProgress));
}

#[tokio::test]
async fn tasks_endpoint_returns_persisted_status() {
    let temp = tempdir().expect("failed to create temp dir");
    let spec_root = temp.path().to_path_buf();
    write_tasks_files(&spec_root, "TASK-003", TaskStatus::Review);

    let repo = Arc::new(adapters_spec_fs::FsGovernanceRepository::new(spec_root.join("specs")));
    let app = app_with_workspace_root(repo, spec_root.clone());

    let request = Request::builder()
        .method("GET")
        .uri("/platform/tasks")
        .body(Body::empty())
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("service should not fail");
    let status = response.status();
    let body =
        BodyExt::collect(response.into_body()).await.map(|c| c.to_bytes()).unwrap_or_default();

    if !status.is_success() {
        panic!(
            "expected success status, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }

    let tasks: TasksResponse =
        serde_json::from_slice(&body).expect("failed to deserialize tasks response");

    assert_eq!(tasks.tasks.len(), 1);
    assert_eq!(tasks.tasks[0].status, "Review");
}

```

