bootstrap:
	nix develop -c ./bootstrap-tools.sh

# Install audit tools (cargo-audit, cargo-deny) - run once per machine
audit-tools:
	nix develop -c cargo install --locked cargo-audit cargo-deny

# Run security audit (requires audit-tools)
audit:
	nix develop -c cargo xtask audit

check:
	cargo xtask check

bdd:
	cargo xtask bdd

selftest:
	cargo xtask selftest

skills-lint:
	cargo xtask skills-lint

skills-fmt:
	cargo xtask skills-fmt

policy-test:
	cargo xtask policy-test

ac-status:
	cargo xtask ac-status

bundle task='implement_ac':
	cargo xtask bundle TASK={{task}}

quickstart:
	cargo xtask quickstart

deploy env='dev':
	cargo xtask deploy ENV={{env}}
