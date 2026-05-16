//! Contextual documentation lookup for xtask commands.

use anyhow::Result;
use colored::Colorize;

use crate::cli::Commands;
use crate::devex;

/// Returns list of all available xtask command names
/// Used by devex contract check to validate required commands exist
pub fn all_command_names() -> Vec<&'static str> {
    vec![
        "ac-coverage",
        "ac-ensure-kernel-mapped",
        "ac-history",
        "ac-lint",
        "ac-new",
        "ac-report",
        "ac-slo",
        "ac-status",
        "ac-suggest-scenarios",
        "ac-tests",
        "adr-check",
        "adr-new",
        "agents-fmt",
        "agents-lint",
        "audit",
        "bdd",
        "build-time-capture",
        "build-time-compare",
        "bundle",
        "check",
        "check-api-diff",
        "check-json-schemas",
        "check-layering",
        "check-openapi-diff",
        "ci-local",
        "clean",
        "config-validate",
        "contracts-check",
        "contracts-fmt",
        "coverage",
        "deploy",
        "design-new",
        "dev-up",
        "doctor",
        "docs-check",
        "docs-frontmatter-sync",
        "env-mode",
        "fmt-all",
        "fork-list",
        "fork-register",
        "friction-gh-create",
        "friction-gh-link",
        "friction-list",
        "friction-new",
        "friction-resolve",
        "graph-export",
        "hakari",
        "help-flows",
        "idp-check",
        "idp-snapshot",
        "install-hooks",
        "issues-search",
        "kernel-check",
        "kernel-pack",
        "kernel-smoke",
        "kernel-status",
        "migrate",
        "pin-actions",
        "policy-test",
        "pr-cover",
        "pr-update",
        "precommit",
        "publish-check",
        "question-new",
        "question-resolve",
        "questions-list",
        "quickstart",
        "receipts-economics",
        "receipts-forensic",
        "receipts-gate",
        "receipts-quality",
        "receipts-telemetry",
        "receipts-timeline",
        "receipts-validate",
        "release-bundle",
        "release-prepare",
        "release-verify",
        "sbom-local",
        "selftest",
        "service-descriptor",
        "service-init",
        "skills-fmt",
        "skills-lint",
        "spellcheck",
        "status",
        "suggest-next",
        "task-create",
        "task-update",
        "tasks-list",
        "test-ac",
        "test-changed",
        "tools-checksum-update",
        "tools-checksum-verify",
        "ui-contract-check",
        "version",
        "version-check",
    ]
}

pub(crate) fn show_command_docs(command: &Commands) -> Result<()> {
    let name = get_command_name(command);
    println!();
    println!("{} {}", "Documentation for:".bold(), name.cyan());
    println!("{}", "=".repeat(20 + name.len()).blue());
    println!();

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();
    let spec_path = root.join("specs/devex_flows.yaml");

    if let Ok(spec) = devex::load_spec(&spec_path) {
        if let Some(cmd_spec) = spec.commands.get(name) {
            println!("{}", cmd_spec.summary);
            println!();
            println!("{}: {}", "Category".bold(), cmd_spec.category);

            // Find flows containing this command
            let mut flows: Vec<_> =
                spec.flows.iter().filter(|(_, f)| f.steps.iter().any(|s| s == name)).collect();
            flows.sort_by_key(|(id, _)| *id);

            if !flows.is_empty() {
                println!();
                println!("{}", "Part of flows:".bold());
                for (flow_id, flow) in flows {
                    println!("  • {} ({})", flow.name.cyan(), flow_id);
                    for doc in &flow.documented_in {
                        println!("    → {}", doc.dimmed());
                    }
                }
            }
        } else {
            println!("No specific documentation found in devex_flows.yaml for '{}'", name);
        }
    } else {
        println!("Warning: Failed to load specs/devex_flows.yaml");
    }

    // Always link to glossary
    println!();
    println!("{}", "See also:".bold());
    println!("  • {}", "docs/GLOSSARY.md".blue());
    println!("  • {}", "docs/AGENT_GUIDE.md".dimmed());

    Ok(())
}

fn get_command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Doctor => "doctor",
        Commands::DevUp => "dev-up",
        Commands::InstallHooks => "install-hooks",
        Commands::CiLocal => "ci-local",
        Commands::KernelCheck { .. } => "kernel-check",
        Commands::KernelPack { .. } => "kernel-pack",
        Commands::KernelSmoke => "kernel-smoke",
        Commands::KernelStatus => "kernel-status",
        Commands::Selftest => "selftest",
        Commands::Quickstart => "quickstart",
        Commands::Check => "check",
        Commands::Precommit { .. } => "precommit",
        Commands::TestChanged { .. } => "test-changed",
        Commands::CheckApiDiff { .. } => "check-api-diff",
        Commands::CheckOpenapiDiff => "check-openapi-diff",
        Commands::CheckJsonSchemas { .. } => "check-json-schemas",
        Commands::CheckLayering => "check-layering",
        Commands::AcStatus { .. } => "ac-status",
        Commands::AcNew { .. } => "ac-new",
        Commands::AcCoverage { .. } => "ac-coverage",
        Commands::AcSuggestScenarios { .. } => "ac-suggest-scenarios",
        Commands::AcTests { .. } => "ac-tests",
        Commands::TestAc { .. } => "test-ac",
        Commands::Bdd => "bdd",
        Commands::AcReport { .. } => "ac-report",
        Commands::AcHistory { .. } => "ac-history",
        Commands::AcSlo { .. } => "ac-slo",
        Commands::AcEnsureKernelMapped { .. } => "ac-ensure-kernel-mapped",
        Commands::AcLint { .. } => "ac-lint",
        Commands::AdrNew { .. } => "adr-new",
        Commands::AdrCheck => "adr-check",
        Commands::DesignNew { .. } => "design-new",
        Commands::DocsCheck => "docs-check",
        Commands::DocsFrontmatterSync { .. } => "docs-frontmatter-sync",
        Commands::Spellcheck => "spellcheck",
        Commands::ContractsCheck => "contracts-check",
        Commands::ContractsFmt => "contracts-fmt",
        Commands::UiContractCheck => "ui-contract-check",
        Commands::FrictionList { .. } => "friction-list",
        Commands::FrictionNew { .. } => "friction-new",
        Commands::FrictionResolve { .. } => "friction-resolve",
        Commands::FrictionGhCreate { .. } => "friction-gh-create",
        Commands::FrictionGhLink { .. } => "friction-gh-link",
        Commands::QuestionsList { .. } => "questions-list",
        Commands::QuestionNew { .. } => "question-new",
        Commands::QuestionResolve { .. } => "question-resolve",
        Commands::IssuesSearch { .. } => "issues-search",
        Commands::ForkList { .. } => "fork-list",
        Commands::ForkRegister { .. } => "fork-register",
        Commands::SkillsFmt => "skills-fmt",
        Commands::SkillsLint => "skills-lint",
        Commands::AgentsFmt => "agents-fmt",
        Commands::AgentsLint => "agents-lint",
        Commands::TasksList => "tasks-list",
        Commands::TaskCreate { .. } => "task-create",
        Commands::TaskUpdate { .. } => "task-update",
        Commands::SuggestNext(..) => "suggest-next",
        Commands::ReleasePrepare { .. } => "release-prepare",
        Commands::ReleaseBundle { .. } => "release-bundle",
        Commands::ReleaseVerify => "release-verify",
        Commands::SbomLocal => "sbom-local",
        Commands::PublishCheck { .. } => "publish-check",
        Commands::PrCover { .. } => "pr-cover",
        Commands::PrUpdate { .. } => "pr-update",
        Commands::ReceiptsGate { .. } => "receipts-gate",
        Commands::ReceiptsEconomics { .. } => "receipts-economics",
        Commands::ReceiptsValidate { .. } => "receipts-validate",
        Commands::ReceiptsQuality { .. } => "receipts-quality",
        Commands::ReceiptsTelemetry { .. } => "receipts-telemetry",
        Commands::ReceiptsTimeline { .. } => "receipts-timeline",
        Commands::ReceiptsForensic { .. } => "receipts-forensic",
        Commands::ServiceInit { .. } => "service-init",
        Commands::ServiceDescriptor { .. } => "service-descriptor",
        Commands::ConfigValidate { .. } => "config-validate",
        Commands::Audit => "audit",
        Commands::PolicyTest => "policy-test",
        Commands::Coverage => "coverage",
        Commands::BuildTimeCapture => "build-time-capture",
        Commands::BuildTimeCompare { .. } => "build-time-compare",
        Commands::Bundle { .. } => "bundle",
        Commands::HelpFlows => "help-flows",
        Commands::FmtAll => "fmt-all",
        Commands::ToolsChecksumUpdate => "tools-checksum-update",
        Commands::ToolsChecksumVerify => "tools-checksum-verify",
        Commands::Clean => "clean",
        Commands::GraphExport { .. } => "graph-export",
        Commands::Hakari => "hakari",
        Commands::Migrate => "migrate",
        Commands::PinActions => "pin-actions",
        Commands::Deploy { .. } => "deploy",
        Commands::Status => "status",
        Commands::Version { .. } => "version",
        Commands::VersionCheck { .. } => "version-check",
        Commands::EnvMode { .. } => "env-mode",
        Commands::IdpSnapshot { .. } => "idp-snapshot",
        Commands::IdpCheck => "idp-check",
    }
}
