bootstrap:
	nix develop -c ./bootstrap-tools.sh

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