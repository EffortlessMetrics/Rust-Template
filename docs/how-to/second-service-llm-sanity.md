---
title: Add a second service and sanity-check LLM flows
status: draft
last_updated: 2025-11-25
tags: [adoption, llm, devex]
---

This is the quickest path to exercise the template a second time and verify the LLM/agent flows stay healthy.

## Steps

1) **Clone the cell and retarget metadata**

- `git clone` this repo to a new directory (e.g., `rust-template-demo-2`).
- Update `specs/service_metadata.yaml` (service id/name/description/tags), `specs/spec_ledger.yaml` template_version if needed, and `README.md` branding.

2) **Adjust ACs/requirements for a tiny domain slice**

- Use `cargo xtask ac-new ...` to add one or two domain ACs.
- Wire minimal domain logic (small handler + model change) to satisfy the new ACs.

3) **Run the governed loop**

```bash
cargo xtask dev-up
cargo xtask test-changed
cargo xtask test-ac AC-...
cargo xtask selftest
cargo xtask release-bundle 0.1.0
```

4) **Exercise with Skills/agents**

- From the clone, run the same flow via .claude Skills or your agent harness.
- Capture any friction in `FRICTION_LOG.md` and feed improvements back into this template.

5) **LLM sanity**

- Ensure `/platform/schema`, `/platform/status`, and `/platform/tasks` stay aligned with the new ACs.
- Confirm selftest + AC tags still gate the build (including the new optional platform auth/log-hygiene checks).

The goal is to prove the template scales to a second service and that the agent-facing surfaces remain predictable without manual tweaks.
