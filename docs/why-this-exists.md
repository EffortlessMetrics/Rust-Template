# Why This Template Exists
<!-- doclint:disable orphan-version -->

**Version**: v2.4.0
**Last Updated**: 2025-11-19

---

## 1. The Problem This Tries to Solve

Most teams building Rust services today end up with one of these:

- A "starter repo" that bootstraps Axum + a few crates, but:
  - CI is an afterthought.
  - Policies live in wiki pages, not code.
  - LLM usage is "open the repo and pray the model guesses context".
- Or a hand-rolled internal template that:
  - Drifts away from CI over time.
  - Has undocumented invariants ("never touch that job, it's fragile").
  - Is hard to evolve because no one remembers why it looks the way it does.

Three converging pressures make that brittle:

1. **Multi-service Rust** - orgs want repeatable service patterns, not one-offs.
2. **Governance expectations** - security, privacy, infra, and auditors all care about how things are built and changed.
3. **LLM-era development** - engineers are starting to offload work to models, but most repos give the model zero guardrails.

The usual outcome: everybody "does their best", CI is green enough, but:

- You can't see **which acceptance criteria are actually enforced**.
- You can't express **policy as code** without bolting it on later.
- You can't safely let an LLM make non-trivial changes.

This template exists to give a **strong, opinionated "yes"** to those three pressures in one place.

---

## 2. What This Template Actually Is

At a high level, this is:

> A Rust service template + library that bakes in
> spec-as-code, policy-as-code, LLM-native workflows, and a Nix dev environment
> so dev/CI/governance all align by default and agents can work safely at full speed.

Concretely, it provides:

- **Spec-as-code**
  - `specs/spec_ledger.yaml` - story -> requirement -> AC mapping
  - `specs/features/*.feature` - BDD scenarios tagged with `@AC-XXX`
  - `docs/feature_status.md` - generated AC coverage report

- **Policy-as-code**
  - `policy/*.rego` - K8s, flags, privacy, LLM bundle rules
  - `policy/testdata/*.yaml/json` - fixtures proving "good" vs "bad" configs
  - `xtask policy-test` - conftest-based validation (strict in CI, optional locally)

- **LLM-native, governance-bounded**
  - `.llm/contextpack.yaml` - declarative bundles of relevant files
  - `xtask bundle <task>` - generates bounded, structured context for agents
  - `.claude/skills/*` - repo-specific Skills that encode how agents should use
    `xtask`, specs, and `/platform/*` APIs
  - Agents are expected to:
    - create new AC IDs and scenarios,
    - propose policy changes,
    - and touch infra/CI configs.
  - The guardrail is not "never let the LLM do X." The guardrail is:
    - every change must still deserialize into the spec structs,
    - pass BDD, policy, and graph invariants via `xtask selftest`,
    - and show up in `/platform/status` and the governance graph.

- **Nix-first development environment**
  - `flake.nix` - devshell that matches CI (Rust toolchain, conftest, yq, jq...)
  - `nix develop` - one command to get a CI-equivalent environment
  - Selftest behaves differently inside vs outside devshell:
    - Outside Nix: policy tests skipped with a clear hint to use Nix.
    - Inside Nix/CI: all policy tests enforced.

- **Orchestrated developer workflow**
  - `xtask selftest` - single entrypoint that runs:
    1. fmt + clippy + tests
    2. BDD scenarios
    3. AC status mapping
    4. LLM bundle sanity
    5. Policy tests (when conftest is available)

> For a developer, the mental model is:
> **"Run `nix develop` once, then `cargo run -p xtask -- selftest` before you trust anything."**

---

## 3. Who This Is For (And Who It's Not For)

### 3.1. Intended Users

This template is opinionated and heavyweight **on purpose**. It's meant for:

- **Platform / DevEx groups** who want:
  - A **canonical Rust service skeleton** with governance baked in.
  - A way to prove "this is how we build services here".
  - A starting point they can fork, trim, or extend.

- **Teams using LLMs seriously** as part of delivery:
  - They want models to touch real code paths,
  - But they need traceability and guardrails,
  - And they can't afford "the AI quietly changed a security boundary".

- **Consultancies / boutiques** who:
  - Build internal platforms for clients,
  - Need a credible, inspectable reference implementation,
  - Want to show judgment in architecture + governance, not just "we used Rust".

### 3.2. Not a Good Fit For

You probably don't want this template if:

- You're prototyping and don't want any governance yet.
- You're building a single toy service with no intention of reuse.
- Your org cannot or will not accept:
  - Nix (or at least pinned toolchains),
  - Rego,
  - BDD-style specs.

You can still use the **library crates** (`rust_iac_xtask_core`, `rust_iac_config`) without adopting the full template, but the repo itself assumes you *care* about these things.

### 3.3 LLM-native vs human-first teams

This template is heavy on structure and boilerplate by design:
- spec ledgers,
- AC IDs and feature files,
- tasks and devex flows,
- policies and governance graph invariants.

For an all-human team, that can feel like overhead.

For an LLM-native team, it is fuel:
- agents are good at filling in structured boilerplate,
- humans are good at deciding which structures and trade-offs matter.

This repo is designed to shift humans from "vibe coding" to "vibe architecting":
you shape stories, requirements, and policies; agents fill in scaffolding and
code under `xtask selftest` and the `/platform/*` contract.

---

## 4. Design Principles

This template is built around a few principles:

1. **Spec -> Tests -> Code (AC-first)**
   ACs live in the ledger and features first. Code follows.
   You should always be able to answer:
   - "Which AC does this test prove?"
   - "Which AC does this code path exist for?"

2. **Policies Are Code, Not Slides**
   K8s, flags, privacy, and LLM bundle behavior are defined in Rego and tested.
   If a requirement isn't encoded, assume it doesn't exist.

3. **Nix-First, Not Nix-Only**
   The primary path is:

   ```bash
   nix develop
   cargo run -p xtask -- selftest
   ```

   But:
   - Library crates don't depend on Nix.
   - Local dev without Nix is supported (with reduced guarantees).
   - CI always runs **inside** the pinned Nix environment.

4. **LLM-Native, Kernel-Governed**

This repo is built for agents to do real work, not just autocomplete.

Agents are allowed to:

- Propose new stories, requirements, AC IDs, and tasks.
- Edit specs, docs, policies, and even infra/CI wiring.
- Drive the inner loop from “we saw a signal” to “there is a branch and a draft PR”.

We do not rely on “the LLM is careful”. We rely on the kernel:

- Specs must deserialize into Rust structs and pass schema validation.
- Policies must pass OPA/Rego tests against known-good and known-bad fixtures.
- Governance graph invariants must hold (no orphaned requirements, missing ACs, unreachable commands).
- Unit tests and BDD scenarios must pass.
- `xtask selftest` must be green in a Tier 1 environment.
- A human still owns the final merge decision.

Agents have write access. The governance kernel has veto power.

5. **Selftest as a Contract**
   `xtask selftest` is the contract between:
   - Developers,
   - Platform/DevEx,
   - CI/CD,
   - Security/governance.

   If selftest is green (in the devshell), the template is behaving as designed.

### 4.5 LLM-Assisted vs LLM-Native

You can use this template in a classic “LLM-assisted” way:

- Humans edit specs and code.
- Models help draft tests, docs, and refactors.
- `xtask selftest` keeps everyone honest.

But the design assumes “LLM-native” work:

- Swarms of agents run the full flow from signal -> problem -> plan -> branch -> draft PR.
- Humans shape problems, choose trade-offs, and approve merges.
- Boilerplate is a feature, not a tax: models fill out structured specs, ACs, and tasks quickly, and the kernel enforces that everything lines up.

The very structure that feels heavy to a manual-only team is what makes agent-native development safe. It turns vibe coding into vibe architecting.

---

## 5. Where This Sits in the IDP Landscape

### 5.1 Cell vs Portal vs Orchestrator

- This repo is a governed, LLM-native Rust cell: per-service specs, policies, graph invariants, `/platform/*` introspection, and agent-safe bundles (`xtask bundle`, `/platform/tasks/suggest-next`) plus `.claude/skills/*`.
- It is not a portal (no fleet view, scorecards, or multi-service catalog) and not an orchestrator (no environment or infra wiring). It composes under portals and above orchestrators.

### 5.2 Practical Alternatives Teams Choose

- **Portal + thin templates** (Backstage/Port/OpsLevel + Axum cookiecutter): Pros - multi-language, instant catalog and scorecards. Cons - no per-repo governance kernel; specs/ACs/docs/policies drift independently; no graph invariants; agent surface is "whatever is in the repo."
- **Platform orchestrator + bare repo** (Humanitec or similar): Pros - runtime wiring and environment management. Cons - each repo is still ad hoc; governance inside the repo is out of scope. Complementary if each service exposes this cell surface (`/platform/status`, `ac-status`, policy tests).
- **Just a Rust starter**: Fastest to "hello world + CI." Missing spec ledger, AC traceability, policies, graph invariants, and agent ergonomics.

### 5.3 One-Line Comparisons

- Versus portals: portals give fleet scorecards; this gives a governed Rust cell with `/platform/status` and `xtask selftest` as the contract.
- Versus orchestrators: orchestrators deploy and wire environments; this makes each deployable thing self-describing and governed.
- Versus generic templates: they scaffold; this enforces specs -> ACs -> tests -> docs -> policies and exposes them to humans and agents.

---

### 5.4 Agent-Native Cells in a Human Platform

Most IDP stories today are human-first:

- Portals give humans scorecards and catalogs.
- Orchestrators give humans environment wiring.
- Templates give humans a starting point.

This template assumes a mixed world:

- Humans own direction, risk, and merge decisions.
- Agents own most of the mechanical work inside a single cell.
- The kernel (`xtask selftest`, policies, graph invariants, AC coverage) is the referee between them.

From a platform point of view, this repo is a unit of capacity you can hand to a swarm: "Here is the governed box you are allowed to change. If you keep selftest green, your work is admissible."

## 6. How This Fits With Your Orchestration Layer

This template is designed to play nicely with an orchestration layer that can:

- Spin up new services from the template (greenfield),
- Inject governance into existing services via the library crates (brownfield),
- Coordinate LLM workflows across repos using:
  - .llm/contextpack.yaml,
  - standardized prompts from CLAUDE.md,
  - xtask bundle/selftest as well-defined entrypoints.

In other words, this repo is the **"governed cell"** your orchestrator can stamp out:

- Nix dev env -> no drift between dev and CI.
- xtask commands -> stable automation surface.
- Specs/policies -> machine-readable governance.

Your orchestration layer doesn't have to guess; it can:

- Call `xtask bundle` to prepare LLM work.
- Call `xtask selftest` to validate outputs.
- Enforce that new services **start** from a known-good baseline.

## 7. Agent Flows: How Swarms Use This Repo

Agents do not get a single giant prompt. They move work through four flows that line up with the governed surfaces in this repo:

1. **Signal -> Problem -> Requirements**

   Entry: noisy issue, Slack thread, support ticket.
   Exit: a clean problem statement, scoped requirements, and ACs in the ledger.

   Agents:

   - Canonicalize issues into structured problems.
   - Propose or update REQs and ACs in `specs/spec_ledger.yaml`.
   - Draft BDD scenarios in `specs/features/*.feature`.

2. **Requirements -> Design -> Plan**

   Entry: problem statement and requirements.
   Exit: ADRs, design docs, and an implementation plan.

   Agents:

   - Draft ADRs under `docs/adr/`, tied back to REQs and ACs.
   - Sketch designs under `docs/design/`.
   - Produce an implementation plan and test inventory that a human can review.

3. **Plan -> Branch -> Draft PR**

   Entry: implementation plan and test inventory.
   Exit: a branch with code and tests, plus a draft PR.

   Agents:

   - Use `xtask bundle` and `.llm/contextpack.yaml` to stay in-bounds.
   - Extend BDD scenarios and unit tests.
   - Implement code to satisfy those tests.
   - Open a draft PR that links back to the issue and design.

4. **Draft PR -> Reviewed -> Merged -> Verified**

   Entry: draft PR.
   Exit: a merge recommendation and verification notes.

   Agents:

   - Run `xtask selftest` and read its output.
   - Check AC coverage, policy tests, and graph invariants.
   - Summarize risks and open questions in the PR.
   - Propose a merge decision; humans still own the merge button.

These flows are just structured ways of using the same kernel surfaces humans use today. The difference is that agents handle the boilerplate; humans spend their time on shaping, design, and risk.

---

## 8. How to Position This Externally

When you put this on your public profile / GitHub:

You're not just showing "I can write Rust". You're showing:

- I can **define a platform standard** for Rust services.
- I understand **how governance, CI, and LLMs intersect**.
- I can **land it in a repo** with:
  - clear docs,
  - a stable CLI surface (xtask),
  - and a documented release history (v2.0.0-v2.3.0).

For a Head of Platform / Head of DevEx audience, that's the difference between:

- "We use Rust + some tools" vs.
- "We have a pattern for safe, fast AI-assisted development in Rust, and here is a working example."

---

## 9. How to Use This Doc

You can point people to this file as:

- The **"why this exists"** narrative,
- A companion to `README.md` (which stays more practical),
- A foundation for:
  - conference talks,
  - blog posts,
  - or client-facing primers.

If you need to adapt it for a specific client or org, you can fork this into a `WHY_TEMPLATE_FITS_<CLIENT>.md` with concrete examples.
