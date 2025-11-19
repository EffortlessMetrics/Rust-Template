# Nix-first dev helpers

# Enter devshell
dev:
    nix develop

# Fast checks (fmt + clippy + tests)
check:
    nix develop -c cargo run -p xtask -- check

# Full validation suite
selftest:
    nix develop -c cargo run -p xtask -- selftest

# BDD only
bdd:
    nix develop -c cargo run -p xtask -- bdd

# Policy tests only
policy-test:
    nix develop -c cargo run -p xtask -- policy-test

# AC status report
ac-status:
    nix develop -c cargo run -p xtask -- ac-status

# Generate LLM bundle for implementing ACs
bundle:
    nix develop -c cargo run -p xtask -- bundle implement_ac
