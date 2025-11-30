use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ContextPack {
    tasks: std::collections::HashMap<String, Task>,
}

#[derive(Debug, Deserialize)]
struct Task {
    max_bytes: usize,
    include: Vec<String>,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleManifest {
    bundle_version: i32,
    task_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requirement_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ac_ids: Vec<String>,
    git_sha: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specs: Vec<ManifestSpec>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    docs: Vec<ManifestDoc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tests: Vec<ManifestTest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestSpec {
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestDoc {
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestTest {
    r#type: String,
    tag: String,
    file: String,
}

/// Generate LLM context bundle for a task
pub fn run(task_name: &str) -> Result<()> {
    let workspace_root = get_workspace_root()?;
    let contextpack_path = workspace_root.join(".llm/contextpack.yaml");

    if !contextpack_path.exists() {
        anyhow::bail!("ContextPack not found: {}", contextpack_path.display());
    }

    println!("Loading contextpack: {}", contextpack_path.display());
    let contextpack = load_contextpack(&contextpack_path)?;

    let task = contextpack.tasks.get(task_name).with_context(|| {
        let available: Vec<_> = contextpack.tasks.keys().map(|s| s.as_str()).collect();
        format!(
            "Task '{}' not found in contextpack. Available tasks: {}",
            task_name,
            available.join(", ")
        )
    })?;

    println!("Task: {}", task_name.blue());
    if !task.description.is_empty() {
        println!("  {}", task.description);
    }
    println!("  Max bytes: {}", task.max_bytes);
    println!("  Include patterns: {}", task.include.len());

    let git_sha = get_git_sha(&workspace_root)?;
    println!("Git SHA: {}", git_sha);

    println!("Resolving files...");
    let files = resolve_files(&workspace_root, &task.include)?;
    println!("  Found {} matching files", files.len());

    // Create bundle/<TASK>/ directory structure
    let bundle_root = workspace_root.join("bundle");
    fs::create_dir_all(&bundle_root)?;
    let bundle_task_dir = bundle_root.join(task_name);
    fs::create_dir_all(&bundle_task_dir)?;

    // Build and write context.md
    let context_path = bundle_task_dir.join("context.md");
    println!("Building context: {}", context_path.display());
    let (file_count, total_bytes) =
        build_context(&context_path, task, &files, &git_sha, &workspace_root)?;

    // Create manifest
    let manifest = BundleManifest {
        bundle_version: 1,
        task_id: task_name.to_string(),
        requirement_ids: vec![],
        ac_ids: vec![],
        git_sha,
        timestamp: Utc::now().to_rfc3339(),
        specs: vec![ManifestSpec { file: "specs/spec_ledger.yaml".to_string() }],
        docs: vec![ManifestDoc { file: "docs/explanation/TEMPLATE-CONTRACTS.md".to_string() }],
        tests: vec![],
    };

    // Write manifest.yaml
    let manifest_path = bundle_task_dir.join("bundle.yaml");
    let manifest_yaml =
        serde_yaml::to_string(&manifest).context("Failed to serialize manifest to YAML")?;
    fs::write(&manifest_path, manifest_yaml).context("Failed to write manifest")?;

    println!("{} Generated {}", "[OK]".green(), bundle_task_dir.display());
    println!("  Files: {}", file_count);
    println!("  Size: {} bytes", total_bytes);
    println!("  Manifest: {}", manifest_path.display());

    if total_bytes > task.max_bytes {
        println!("  {} Size limit exceeded!", "[WARN]".yellow());
    }

    Ok(())
}

fn get_workspace_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to run git rev-parse")?;

    if !output.status.success() {
        anyhow::bail!("Not in a git repository");
    }

    let root = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(root))
}

fn load_contextpack(path: &Path) -> Result<ContextPack> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read contextpack: {}", path.display()))?;

    let contextpack: ContextPack = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse contextpack YAML: {}", path.display()))?;

    Ok(contextpack)
}

fn get_git_sha(workspace_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to get git SHA")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git SHA");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn resolve_files(workspace_root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let llmignore = load_llmignore(workspace_root)?;
    let mut seen = HashSet::new();
    let mut files = Vec::new();

    for pattern in patterns {
        let output = Command::new("git")
            .current_dir(workspace_root)
            .args(["ls-files", pattern])
            .output()
            .with_context(|| format!("Failed to run git ls-files for pattern: {}", pattern))?;

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            let path_str = line.trim();
            if path_str.is_empty() {
                continue;
            }

            // Skip if already seen
            if seen.contains(path_str) {
                continue;
            }

            // Skip if matches .llmignore patterns (using gitignore semantics)
            let path = Path::new(path_str);
            if llmignore.matched(path, false).is_ignore() {
                continue;
            }

            let full_path = workspace_root.join(path_str);
            if full_path.exists() && full_path.is_file() {
                seen.insert(path_str.to_string());
                files.push(PathBuf::from(path_str));
            }
        }
    }

    Ok(files)
}

/// Load .llmignore file and build gitignore matcher
fn load_llmignore(workspace_root: &Path) -> Result<ignore::gitignore::Gitignore> {
    let ignore_path = workspace_root.join(".llm/.llmignore");

    let mut builder = ignore::gitignore::GitignoreBuilder::new(workspace_root);

    if ignore_path.exists() {
        builder.add(&ignore_path);
    }

    builder.build().context("Failed to build .llmignore matcher")
}

fn build_context(
    context_path: &Path,
    task: &Task,
    files: &[PathBuf],
    git_sha: &str,
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut output = String::new();

    // Header
    output.push_str("# LLM Context Bundle\n\n");
    output.push_str(&format!("**Git SHA:** {}\n\n", git_sha));
    if !task.description.is_empty() {
        output.push_str(&format!("**Description:** {}\n\n", task.description));
    }
    output.push_str(&format!("**Max bytes:** {}\n\n", task.max_bytes));
    output.push_str("---\n\n");

    let mut total_bytes = output.len();
    let mut file_count = 0;

    for file_path in files {
        let full_path = workspace_root.join(file_path);

        match fs::read_to_string(&full_path) {
            Ok(content) => {
                let file_section =
                    format!("# FILE: {}\n\n```\n{}\n```\n\n", file_path.display(), content);
                let new_total = total_bytes + file_section.len();

                if new_total > task.max_bytes {
                    eprintln!(
                        "  {} Size limit reached, skipping remaining files",
                        "[WARN]".yellow()
                    );
                    break;
                }

                output.push_str(&file_section);
                total_bytes = new_total;
                file_count += 1;
            }
            Err(e) => {
                eprintln!("  {} Failed to read {}: {}", "[WARN]".yellow(), file_path.display(), e);
            }
        }
    }

    fs::write(context_path, output)
        .with_context(|| format!("Failed to write context: {}", context_path.display()))?;

    Ok((file_count, total_bytes))
}
