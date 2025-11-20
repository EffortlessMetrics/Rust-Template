use anyhow::Result;
use colored::Colorize;

/// Print a flow-based map of xtask commands
pub fn run() -> Result<()> {
    println!("{}", "FLOWS & COMMAND GROUPS".bold());
    println!();

    println!("{}", "🚀 Onboarding (New Developer / Machine)".cyan().bold());
    println!("  {}  Validate environment (Rust, Nix, tools)", "doctor         ".bold());
    println!("  {}  Fast dev loop (fmt + clippy + tests)", "check          ".bold());
    println!("  {}  Full gate (BDD + AC + policies)", "selftest       ".bold());
    println!();

    println!("{}", "✨ Design & Acceptance Criteria".cyan().bold());
    println!("  {}  Create architecture decision record", "adr-new        ".bold());
    println!("  {}  Validate ADR references and linkage", "adr-check      ".bold());
    println!("  {}  Scaffold new acceptance criterion", "ac-new         ".bold());
    println!("  {}  Generate AC coverage/status report", "ac-status      ".bold());
    println!("  {}  Generate LLM context bundle (e.g., implement_ac)", "bundle         ".bold());
    println!("  {}  Run BDD acceptance tests", "bdd            ".bold());
    println!();

    println!("{}", "🔒 Security & Dependencies".cyan().bold());
    println!("  {}  Security & license audit (cargo-audit + cargo-deny)", "audit          ".bold());
    println!("  {}  Generate local SPDX SBOM for the workspace", "sbom-local     ".bold());
    println!();

    println!("{}", "📦 Release Management".cyan().bold());
    println!("  {}  Bump versions & seed changelog entry", "release-prepare".bold());
    println!("  {}  Pre-release gate (selftest + audit + docs)", "release-verify ".bold());
    println!();

    println!("{}", "📚 Documentation & Consistency".cyan().bold());
    println!("  {}  Verify version alignment & generated docs", "docs-check     ".bold());
    println!();

    println!("{}", "🛠️  Infrastructure & Maintenance".cyan().bold());
    println!("  {}  Deploy to environment", "deploy         ".bold());
    println!("  {}  Run database migrations", "migrate        ".bold());
    println!("  {}  Manage workspace dependency unification", "hakari         ".bold());
    println!("  {}  Pin GitHub Actions to SHAs", "pin-actions    ".bold());
    println!("  {}  Clean workspace artifacts (target/, etc.)", "clean          ".bold());
    println!("  {}  Format Rust and related artifacts", "fmt-all        ".bold());
    println!("  {}  Run Rego/OPA policy tests", "policy-test    ".bold());
    println!("  {}  Minimal smoke-check of the template", "quickstart     ".bold());
    println!();

    println!("{}", "🔍 Meta".cyan().bold());
    println!("  {}  Show this flow-based command map", "help-flows     ".bold());
    println!();

    println!("{}", "For full details:".dimmed());
    println!("  • {}", "README.md → Developer Workflows".dimmed());
    println!("  • {}", "CLAUDE.md → Golden Path Workflows".dimmed());
    println!("  • {}", "CONTRIBUTING.md → Common Workflows".dimmed());

    Ok(())
}
