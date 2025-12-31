---
id: HOW-TO-ADOPT-KERNEL-001
title: "Adopt the Rust-as-Spec Kernel (v3.3.9-kernel)"
doc_type: how_to
version: 3.3.14
last_updated: 2025-12-31
stories: [US-TPL-PLT-001]
requirements: [REQ-TPL-PLATFORM-INTROSPECTION]
acs: [AC-TPL-KERNEL-CONTRACT-EMITTED, AC-TPL-AC-STATUS-CONSISTENCY]
adrs: [ADR-0005]
---

<!-- doclint:disable orphan-version -->

# How to Adopt the Rust-as-Spec Kernel

This guide shows how to start a **new service** from the kernel baseline
tagged as **`v3.3.9-kernel`**.

You'll:

- Clone the kernel template
- Validate the environment and governance gates
- Change service identity using `service-init`
- Confirm that introspection surfaces and selftest are green

> If you're in a hurry, see the **TL;DR** below and come back for details.

---

## 0. Preconditions

You should have:

- `git` installed
- Nix with flakes enabled (for Tier-1, recommended)
- A POSIX shell (Linux/macOS/WSL2). Native Windows works as Tier-2 (see Environment docs).

---

## 1. TL;DR

```bash
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service
git checkout v3.3.9-kernel

nix develop
cargo xtask dev-up
cargo xtask selftest

cargo xtask service-init \
  --id my-service \
  --name "My Service" \
  --description "What this service does" \
  --tags demo example

cargo xtask selftest
cargo run -p app-http
# → Visit http://localhost:8080/ui
# → curl http://localhost:8080/platform/status
```

If all of that works, you've successfully adopted the kernel. From here you add
your own domain stories/REQs/ACs and services.

---

## 2. Clone the kernel baseline

Start from a clean directory:

```bash
git clone https://github.com/EffortlessMetrics/Rust-Template.git my-service
cd my-service
git checkout v3.3.9-kernel
```

This guarantees you're inheriting the frozen kernel, not a moving `main`.

If you run `git tag -l 'v3.3.9*'` you should see:

```text
v3.3.9-kernel
```

---

## 3. Enter the dev environment and validate

Enter the Nix dev shell (Tier-1 hermetic environment):

```bash
nix develop
```

Then:

```bash
cargo xtask dev-up
cargo xtask selftest
```

You should see:

- All 11 selftest steps green
- Kernel AC summary showing all kernel ACs passing, 0 unknown (run `cargo xtask ac-status --summary` for exact count)

If this is not true on a clean clone of `v3.3.9-kernel`, treat it as a bug and
capture it in a friction entry or issue.

---

## 4. Change service identity

Use `service-init` to give the service its own identity:

```bash
cargo xtask service-init \
  --id my-service \
  --name "My Service" \
  --description "What this service does" \
  --tags demo example
```

This updates:

- `specs/service_metadata.yaml`
- Identity-bearing docs (`README.md`, `CLAUDE.md`, selected docs under `docs/`)
- Optional fork registry entries (depending on configuration)

Re-run:

```bash
cargo xtask selftest
```

Selftest must be green again before you proceed.

---

## 5. Validate introspection surfaces

Start the HTTP app:

```bash
cargo run -p app-http
```

In another shell:

```bash
curl http://localhost:8080/platform/status
curl http://localhost:8080/platform/docs/index
curl http://localhost:8080/platform/tasks
```

And in a browser:

- [http://localhost:8080/ui](http://localhost:8080/ui)

You should see:

- Your new `service_id` and name from `service_metadata.yaml`
- Docs index listing this repo's docs
- Any starter tasks defined in `specs/tasks.yaml`

If `/platform/status` or `/ui` still show the old template identity, stop and
fix that before adding domain behaviour.

---

## 6. Start adding your own domain

From here you:

1. Add new Stories/REQs/ACs **in your own namespace** to `specs/spec_ledger.yaml`
2. Add BDD features tagged with `@AC-...` under `specs/features/*.feature`
3. Add domain code in new crates or modules
4. Use the validation ladder regularly:

   ```bash
   cargo xtask check             # fmt, clippy, unit tests
   cargo xtask test-changed      # changed-only tests
   cargo xtask test-ac AC-XXXX   # single AC
   cargo xtask ac-status         # AC -> test mapping
   cargo xtask selftest          # full governance gate
   ```

Kernel ACs (existing `AC-TPL-*` / `AC-PLT-*` with `must_have_ac: true`) should
remain intact. If you need to change kernel behaviour, that's a kernel evolution
and should be done upstream with:

- An ADR,
- Spec/BDD/test changes,
- And a new `v3.3.9-kernel` or later tag with updated evidence.

---

## 7. Troubleshooting

If you hit issues:

- Environment: see `docs/TROUBLESHOOTING.md` and `docs/reference/environment.md`
- Selftest failures: use `cargo xtask selftest --verbose` and see
  `docs/SELECTIVE_TESTING.md`
- Platform endpoints: verify with `curl http://localhost:8080/platform/schema`

When behaviour doesn't match the docs or contracts, log it:

- For DevEx issues: `cargo xtask friction-new` or edit friction entries
- For unclear specs: file an issue and/or draft an ADR

Those feedback loops are how the kernel evolves for the next tag.

---

## Related Docs

- [KERNEL_SNAPSHOT.md](../KERNEL_SNAPSHOT.md) - What you're inheriting
- [FIRST_FORK.md](./FIRST_FORK.md) - One-page quick start
- [pre-fork-checklist.md](./pre-fork-checklist.md) - Validate before forking
- [new-service-from-template.md](./new-service-from-template.md) - Detailed setup
