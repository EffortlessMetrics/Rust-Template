use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(check: bool) -> Result<()> {
    let root = workspace_root_path()?;
    let out_dir = root.join(RIPR_PR_DIR);

    if check {
        validate_pr_contract(&out_dir)?;
        println!("ripr-pr: output contract is valid");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let ripr_bin = ripr_bin();

    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--format")
        .arg("repo-exposure-json")
        .current_dir(&root)
        .output()
        .with_context(|| format!("failed to execute {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{ripr_bin} repo exposure failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid repo exposure JSON"))?;
    std::fs::write(&json_path, &output.stdout)
        .with_context(|| format!("failed to write {}", json_path.display()))?;
    write_pr_markdown(&md_path)?;
    validate_pr_contract(&out_dir)
}

pub fn run_review_comments(check: bool) -> Result<()> {
    let root = workspace_root_path()?;
    let out_dir = root.join(RIPR_REVIEW_DIR);
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");

    if check {
        validate_json_file(&json_path)?;
        ensure_nonempty(&md_path)?;
        println!("ripr-review-comments: output contract is valid");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();
    let base = std::env::var("RIPR_BASE").unwrap_or_else(|_| "origin/main".to_string());
    let head = std::env::var("RIPR_HEAD").unwrap_or_else(|_| "HEAD".to_string());

    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&root)
        .arg("--base")
        .arg(&base)
        .arg("--head")
        .arg(&head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&root)
        .output()
        .with_context(|| format!("failed to execute {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{ripr_bin} review-comments failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    validate_json_file(&json_path)?;
    if !md_path.exists() {
        std::fs::write(
            &md_path,
            "# RIPR Review Guidance\n\nNo RIPR review guidance was produced.\n",
        )
        .with_context(|| format!("failed to write {}", md_path.display()))?;
    }
    ensure_nonempty(&md_path)
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn workspace_root_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to resolve workspace root from CARGO_MANIFEST_DIR")
}

fn validate_pr_contract(out_dir: &Path) -> Result<()> {
    validate_json_file(&out_dir.join("repo-exposure.json"))?;
    ensure_nonempty(&out_dir.join("repo-exposure.md"))?;
    Ok(())
}

fn validate_json_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("missing required RIPR JSON {}", path.display()))?;
    serde_json::from_str::<Value>(&content)
        .with_context(|| format!("invalid RIPR JSON {}", path.display()))?;
    Ok(())
}

fn ensure_nonempty(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("missing required RIPR Markdown {}", path.display()))?;
    if content.trim().is_empty() {
        anyhow::bail!("RIPR Markdown output is empty: {}", path.display());
    }
    Ok(())
}

fn write_pr_markdown(path: &Path) -> Result<()> {
    let content = "# RIPR PR Evidence\n\nRepo-scoped exposure JSON was generated for this pull request run.\n\nArtifacts are diff-scoped and belong in PR summaries and CI uploads, not README badges.\n";
    std::fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}
