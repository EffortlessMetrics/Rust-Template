use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Mermaid,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReportFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
pub struct GraphExportArgs {
    /// Output format (json or mermaid)
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub format: OutputFormat,

    /// Check graph invariants
    #[arg(long)]
    pub check: bool,

    /// Report format for invariant checks (text or json)
    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    pub report_format: ReportFormat,
}

pub fn run(args: GraphExportArgs) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let specs = spec_runtime::load_all_specs(root)?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)?;

    if args.check {
        let report = spec_runtime::graph::check_invariants(&graph, &specs.devex, &specs.ledger);

        match args.report_format {
            ReportFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            ReportFormat::Text => {
                print_invariant_report(&report);
            }
        }

        if !report.passed {
            anyhow::bail!("Graph invariants failed");
        }

        return Ok(());
    }

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&graph)?);
        }
        OutputFormat::Mermaid => {
            println!("{}", graph.to_mermaid());
        }
    }

    Ok(())
}

fn print_invariant_report(report: &spec_runtime::graph::InvariantReport) {
    eprintln!("Graph Invariants Check ({})", report.checked_at);
    eprintln!();

    for invariant in &report.invariants {
        let status = if invariant.passed { "✓" } else { "✗" };
        eprintln!(
            "  {} {} ({} items checked)",
            status, invariant.description, invariant.checked_count
        );
    }

    if !report.violations.is_empty() {
        eprintln!();
        eprintln!("Violations:");
        for v in &report.violations {
            eprintln!("  - {}", v);
        }
    }

    eprintln!();
    if report.passed {
        eprintln!("✓ All invariants satisfied");
    } else {
        eprintln!("✗ {} violation(s) found", report.violations.len());
    }
}

pub fn run_graph_invariants(_verbosity: u8) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let specs = spec_runtime::load_all_specs(root)?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)?;

    let report = spec_runtime::graph::check_invariants(&graph, &specs.devex, &specs.ledger);

    if !report.passed {
        for v in &report.violations {
            eprintln!("  - {}", v);
        }
        anyhow::bail!("Graph invariants failed");
    }

    Ok(())
}
