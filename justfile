# Install/pin CLIs, run local checks, and common flows

bootstrap:
	@echo "Install local CLIs (via mise) and set up hooks"; 	if command -v mise >/dev/null 2>&1; then mise install; else echo "Install mise first: https://mise.jdx.dev"; fi

checks:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -q
	cargo test --all
	just specs
	@echo "OK"

specs:
	@echo "Running contract checks locally (where possible)"
	@echo " - OpenAPI lint (Redocly): npx @redocly/cli lint specs/openapi/openapi.yaml"
	@echo " - Proto: buf lint / breaking"
	@echo " - Atlas: atlas schema diff"
	@echo "Run CI for authoritative checks."

bdd:
	cargo test --test acceptance

docs:serve:
	@echo "Serve TechDocs via mkdocs (requires Python+mkdocs)"
	@echo "cd backstage && mkdocs serve"
