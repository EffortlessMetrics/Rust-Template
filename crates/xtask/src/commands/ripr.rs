use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fs;
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

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let ripr_bin = ripr_bin();

    let output = run_ripr_repo_exposure(&ripr_bin, &root)?;

    fs::write(&json_path, &output)
        .with_context(|| format!("failed to write {}", json_path.display()))?;
    let parsed: Value = serde_json::from_slice(&output)
        .with_context(|| format!("{ripr_bin} emitted invalid repo exposure JSON"))?;
    fs::write(&md_path, repo_exposure_markdown(&parsed))
        .with_context(|| format!("failed to write {}", md_path.display()))?;

    validate_pr_contract(&out_dir)
}

pub fn run_review_comments(check: bool, base: &str, head: &str) -> Result<()> {
    let root = workspace_root_path()?;
    let out_dir = root.join(RIPR_REVIEW_DIR);

    if check {
        validate_review_contract(&out_dir)?;
        println!("ripr-review-comments: output contract is valid");
        return Ok(());
    }

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let out = out_dir.join("comments.json");
    let ripr_bin = ripr_bin();

    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&root)
        .arg("--base")
        .arg(base)
        .arg("--head")
        .arg(head)
        .arg("--out")
        .arg(&out)
        .current_dir(&root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin} review-comments"))?;

    if !output.status.success() {
        bail!("{ripr_bin} review-comments failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    validate_review_contract(&out_dir)
}

fn run_ripr_repo_exposure(ripr_bin: &str, root: &Path) -> Result<Vec<u8>> {
    let preferred = Command::new(ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(root)
        .arg("--format")
        .arg("repo-exposure-json")
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin} check"))?;

    if preferred.status.success() {
        return Ok(preferred.stdout);
    }

    let fallback = Command::new(ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(root)
        .arg("--base")
        .arg("origin/main")
        .arg("--format")
        .arg("json")
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run fallback {ripr_bin} check"))?;

    if !fallback.status.success() {
        bail!(
            "{ripr_bin} repo exposure failed: preferred format stderr: {}; fallback stderr: {}",
            String::from_utf8_lossy(&preferred.stderr),
            String::from_utf8_lossy(&fallback.stderr)
        );
    }

    Ok(fallback.stdout)
}

fn validate_pr_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    validate_json_file(&json_path)?;
    validate_nonempty_file(&md_path)?;
    Ok(())
}

fn validate_review_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");
    validate_json_file(&json_path)?;
    validate_nonempty_file(&md_path)?;
    Ok(())
}

fn validate_json_file(path: &Path) -> Result<Value> {
    let bytes =
        fs::read(path).with_context(|| format!("missing required file {}", path.display()))?;
    if bytes.is_empty() {
        bail!("required file {} is empty", path.display());
    }
    serde_json::from_slice(&bytes).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn validate_nonempty_file(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("missing required file {}", path.display()))?;
    if text.trim().is_empty() {
        bail!("required file {} is empty", path.display());
    }
    Ok(())
}

fn repo_exposure_markdown(value: &Value) -> String {
    let findings = value.get("findings").and_then(Value::as_array).map_or(0, Vec::len);

    format!(
        "# RIPR PR Evidence\n\n- Findings: `{findings}`\n- Source: `target/ripr/pr/repo-exposure.json`\n\nThis artifact is PR-scoped reviewer evidence and must not be reused as a public README badge.\n"
    )
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
