use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Mermaid,
}

#[derive(Parser, Debug)]
pub struct GraphExportArgs {
    /// Output format (json or mermaid)
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub format: OutputFormat,

    /// Check graph invariants
    #[arg(long)]
    pub check: bool,
}

pub fn run(args: GraphExportArgs) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let specs = spec_runtime::load_all_specs(root)?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)?;

    if args.check {
        if let Err(violations) =
            spec_runtime::graph::check_invariants(&graph, &specs.devex, &specs.ledger)
        {
            for v in violations {
                eprintln!("  - {}", v);
            }
            anyhow::bail!("Graph invariants failed");
        }
        eprintln!("✓ Graph invariants satisfied");
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

pub fn run_graph_invariants(_verbosity: u8) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let specs = spec_runtime::load_all_specs(root)?;
    let graph = spec_runtime::build_graph(&specs.ledger, &specs.devex, &specs.docs)?;

    match spec_runtime::graph::check_invariants(&graph, &specs.devex, &specs.ledger) {
        Ok(_) => Ok(()),
        Err(violations) => {
            for v in violations {
                eprintln!("  - {}", v);
            }
            anyhow::bail!("Graph invariants failed");
        }
    }
}
