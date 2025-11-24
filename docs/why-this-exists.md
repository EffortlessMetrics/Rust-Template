# Why This Template Exists

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
> so dev/CI/governance all align by default.

Concretely, it provides:

- **Spec-as-code**
  - `specs/spec_ledger.yaml` - story -> requirement -> AC mapping
  - `specs/features/*.feature` - BDD scenarios tagged with `@AC-XXX`
  - `docs/feature_status.md` - generated AC coverage report

- **Policy-as-code**
  - `policy/*.rego` - K8s, flags, privacy, LLM bundle rules
  - `policy/testdata/*.yaml/json` - fixtures proving "good" vs "bad" configs
  - `xtask policy-test` - conftest-based validation (strict in CI, optional locally)

- **LLM-native, governance-first**
  - `.llm/contextpack.yaml` – declarative bundles of relevant files
  - `xtask bundle <task>` – generates bounded, structured context for agents
  - `.claude/skills/*` – repo-specific Skills that encode how agents should use
    `xtask`, specs, and `/platform/*` APIs
  - Agents are expected to:
    - create new AC IDs and scenarios,
    - propose policy changes,
    - and touch infra/CI configs.
  - The guardrail is not “never let the LLM do X.” The guardrail is:
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

4. **LLM-native, not unbounded**

Agents are first-class contributors in this repo.

They are allowed to:
- create new AC IDs and BDD scenarios,
- propose changes to policies,
- edit infra and CI configuration.

What they are not allowed to do is bypass the kernel:

- Specs still have to deserialize into Rust structs.
- Graph invariants still have to hold (no orphan REQs/ACs/commands).
- Policies still have to pass Rego tests.
- Kernel ACs still have to meet coverage gates.

If `xtask selftest` is green in a Tier 1 environment, the contract is satisfied,
regardless of whether a human or an agent authored the diff.

If selftest is red, nothing – human or agent – gets a free pass.

5. **Selftest as a Contract**
   `xtask selftest` is the contract between:
   - Developers,
   - Platform/DevEx,
   - CI/CD,
   - Security/governance.

   If selftest is green (in the devshell), the template is behaving as designed.

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

### 5.4 LLM-Native Flows (How Work Actually Moves)

The cell is designed for **LLM-native flows**, not just “AI autocomplete”:

- **Flow 1 – Clarify the signal**  
  Noisy issue or ticket → structured problem + ACs  
  Agents use `spec_ledger.yaml` and BDD templates to turn a Slack thread into a well-formed requirement with tagged scenarios.

- **Flow 2 – Shape the plan**  
  Requirements → ADR + design + test inventory  
  Agents draft ADRs and design docs linked into `specs/` and `docs/`, ready for human edits.

- **Flow 3 – Build the change**  
  Plan → branch + code + tests  
  Agents use `xtask bundle`, `tasks.yaml`, and Skills to open branches, add tests, and implement changes against ACs and flows.

- **Flow 4 – Gate and verify**  
  Draft PR → selftest → evidence bundle  
  Agents run `xtask selftest`, fix what they can, and produce a draft PR plus release evidence. Humans review and merge.

The boilerplate that feels heavy for human-only teams (schemas, specs, tasks, flows) is exactly what makes **agent-heavy teams viable**:

- Agents can move fast on a rich, structured surface.
- Humans spend their time on architecture, risk, and trade-offs, not wiring.

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

---

## 7. How to Position This Externally

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

## 8. How to Use This Doc

You can point people to this file as:

- The **"why this exists"** narrative,
- A companion to `README.md` (which stays more practical),
- A foundation for:
  - conference talks,
  - blog posts,
  - or client-facing primers.

If you need to adapt it for a specific client or org, you can fork this into a `WHY_TEMPLATE_FITS_<CLIENT>.md` with concrete examples.
