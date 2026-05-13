use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let root = workspace_root_path()?;
    let policy = root.join("policy/non-rust-allowlist.toml");
    let content = std::fs::read_to_string(&policy)
        .with_context(|| format!("failed to read {}", policy.display()))?;

    if !content.contains("badges/*.json") {
        anyhow::bail!(
            "policy/non-rust-allowlist.toml must register badges/*.json generated endpoints"
        );
    }

    println!("check-file-policy: non-Rust allowlist covers generated badge endpoints");
    Ok(())
}

fn workspace_root_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .context("failed to resolve workspace root from CARGO_MANIFEST_DIR")
}
