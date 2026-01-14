# Tutorial: Create Your First Service

This tutorial walks you through cloning this template, wiring secrets, and
running the initial CI checks for a new service.

Steps (sketch):
1. Create a new repo from this template.
2. Set owners and base metadata.
3. Configure secrets required by CI (Schema Registry, Apollo, etc.).
4. Run `nix develop`, `cargo run -p xtask -- check`, `cargo run -p xtask -- bdd`.
5. Open a PR and verify that required checks pass.
