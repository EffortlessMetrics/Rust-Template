use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct BrokenLink {
    /// Path to the file containing the broken link
    file: String,
    /// Line number where the link was found
    line: usize,
    /// The link text (what appears in brackets)
    #[allow(dead_code)]
    text: String,
    /// The target URL/path that is broken
    target: String,
    /// Reason the link is broken
    reason: String,
}

/// Validate markdown links in documentation files.
///
/// Scans markdown files in:
/// - docs/
/// - specs/features/ (only .md files)
/// - README.md, CHANGELOG.md, CLAUDE.md (root)
///
/// Checks that local file links (not http/https) resolve to existing files.
/// Anchor links (#section) are advisory warnings only.
pub(crate) fn validate_markdown_links() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    // Regex to match markdown links: [text](url)
    // Captures: group 1 = link text, group 2 = url/path
    let link_re =
        Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").context("Failed to compile markdown link regex")?;

    let mut broken_links: Vec<BrokenLink> = Vec::new();
    let mut anchor_warnings: Vec<BrokenLink> = Vec::new();

    // Collect markdown files to scan
    let mut files_to_scan: Vec<PathBuf> = Vec::new();

    // Root-level markdown files
    for name in ["README.md", "CHANGELOG.md", "CLAUDE.md"] {
        let path = root.join(name);
        if path.exists() {
            files_to_scan.push(path);
        }
    }

    // docs/ directory (recursive)
    let docs_dir = root.join("docs");
    if docs_dir.exists() {
        for entry in WalkDir::new(&docs_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
            e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
        }) {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // specs/features/ directory (only .md files)
    let specs_features_dir = root.join("specs/features");
    if specs_features_dir.exists() {
        for entry in
            WalkDir::new(&specs_features_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
            })
        {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // Process each file
    for file_path in &files_to_scan {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };

        let file_dir = file_path.parent().unwrap_or(root);
        let relative_file = file_path.strip_prefix(root).unwrap_or(file_path).display().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // Skip code blocks (simple heuristic: lines starting with ``` or indented by 4 spaces)
            if line.trim_start().starts_with("```") || line.starts_with("    ") {
                continue;
            }

            for cap in link_re.captures_iter(line) {
                let link_text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let target = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip external URLs
                if target.starts_with("http://") || target.starts_with("https://") {
                    continue;
                }

                // Skip mailto links
                if target.starts_with("mailto:") {
                    continue;
                }

                // Skip pure anchor links (links to sections in the same document)
                if target.starts_with('#') {
                    // These are self-referential anchors, we could validate them
                    // but for now we skip as they're lower priority
                    continue;
                }

                // Handle links with anchors (e.g., "file.md#section")
                let (path_part, anchor_part) = if let Some(hash_pos) = target.find('#') {
                    (&target[..hash_pos], Some(&target[hash_pos + 1..]))
                } else {
                    (target, None)
                };

                // Skip empty paths (just anchor was handled above)
                if path_part.is_empty() {
                    continue;
                }

                // Resolve the target path relative to the file's directory
                let resolved_path = if let Some(stripped) = path_part.strip_prefix('/') {
                    // Absolute path from repo root
                    root.join(stripped)
                } else {
                    // Relative path from file's directory
                    file_dir.join(path_part)
                };

                // Normalize the path (resolve .. and .)
                let normalized_path = normalize_path(&resolved_path);

                // Check if the file/directory exists
                if !normalized_path.exists() {
                    broken_links.push(BrokenLink {
                        file: relative_file.clone(),
                        line: line_num + 1,
                        text: link_text.to_string(),
                        target: target.to_string(),
                        reason: "File not found".to_string(),
                    });
                    continue;
                }

                // If it's a directory link without trailing slash, that's OK
                // (some markdown renderers handle this)

                // If there's an anchor, try to validate it (advisory only)
                // Note: Nested ifs kept for clarity - each level has distinct semantics
                #[allow(clippy::collapsible_if)]
                if let Some(anchor) = anchor_part {
                    if !anchor.is_empty() && normalized_path.is_file() {
                        if let Ok(target_content) = fs::read_to_string(&normalized_path) {
                            if !anchor_exists_in_markdown(&target_content, anchor) {
                                anchor_warnings.push(BrokenLink {
                                    file: relative_file.clone(),
                                    line: line_num + 1,
                                    text: link_text.to_string(),
                                    target: target.to_string(),
                                    reason: format!("Anchor '{}' not found in target file", anchor),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Report results
    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "Broken markdown links:".red().bold());
        for link in &broken_links {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.red(),
                link.reason
            );
        }
    }

    if !anchor_warnings.is_empty() {
        eprintln!();
        eprintln!("{}", "Anchor warnings (advisory):".yellow().bold());
        for link in &anchor_warnings {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.yellow(),
                link.reason
            );
        }
    }

    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Update broken links to point to existing files");
        eprintln!("  • Or remove references to deleted documents");
        eprintln!("  • Check relative paths are correct from the file's location");
        anyhow::bail!("{} broken link(s) found", broken_links.len());
    }

    Ok(())
}

/// Normalize a path by resolving . and .. components.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory markers
            }
            _ => {
                normalized.push(component);
            }
        }
    }
    normalized
}

/// Check if an anchor (heading ID) exists in a markdown file.
///
/// Markdown heading anchors are typically generated by:
/// 1. Converting to lowercase
/// 2. Replacing spaces with hyphens
/// 3. Removing special characters
pub(crate) fn anchor_exists_in_markdown(content: &str, anchor: &str) -> bool {
    let anchor_lower = anchor.to_lowercase();

    for line in content.lines() {
        // Check for markdown headings (# Heading)
        if let Some(heading_text) = line.strip_prefix('#') {
            // Remove leading # and whitespace
            let heading_text = heading_text.trim_start_matches('#').trim();

            // Generate anchor from heading
            let generated_anchor: String = heading_text
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else if c.is_whitespace() {
                        '-'
                    } else {
                        // Skip other characters
                        '\0'
                    }
                })
                .filter(|&c| c != '\0')
                .collect();

            // Remove consecutive hyphens
            let normalized: String =
                generated_anchor.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");

            if normalized == anchor_lower {
                return true;
            }
        }

        // Also check for explicit anchor tags: {#anchor-id}
        if line.contains(&format!("{{#{}}}", anchor)) {
            return true;
        }

        // Check for HTML anchor tags: <a name="anchor-id">
        if line.contains(&format!("name=\"{}\"", anchor))
            || line.contains(&format!("id=\"{}\"", anchor))
        {
            return true;
        }
    }

    false
}
