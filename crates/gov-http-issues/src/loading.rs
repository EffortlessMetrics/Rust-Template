//! Filesystem loading for issue artifact sources.

use std::path::Path;

use gov_http_core::PlatformError;
use gov_http_types::{FrictionEntry, Question};

/// Load all friction entries.
pub(crate) fn load_friction_entries(root: &Path) -> Result<Vec<FrictionEntry>, PlatformError> {
    let friction_dir = root.join("friction");

    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for path in yaml_entry_paths(&friction_dir, "friction")? {
        match load_friction_entry(&path) {
            Ok(friction) => entries.push(friction),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load friction entry"
                );
            }
        }
    }

    Ok(entries)
}

fn load_friction_entry(path: &Path) -> Result<FrictionEntry, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read friction file {}: {}", path.display(), e))
    })?;

    let entry: FrictionEntry = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse friction YAML {}: {}", path.display(), e))
    })?;

    Ok(entry)
}

/// Load all question entries.
pub(crate) fn load_question_entries(root: &Path) -> Result<Vec<Question>, PlatformError> {
    let questions_dir = root.join("questions");

    if !questions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut questions = Vec::new();

    for path in yaml_entry_paths(&questions_dir, "questions")? {
        match load_question_entry(&path) {
            Ok(question) => questions.push(question),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load question entry"
                );
            }
        }
    }

    Ok(questions)
}

fn load_question_entry(path: &Path) -> Result<Question, PlatformError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!("Failed to read question file {}: {}", path.display(), e))
    })?;

    let question: Question = serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!("Failed to parse question YAML {}: {}", path.display(), e))
    })?;

    Ok(question)
}

fn yaml_entry_paths(
    dir: &Path,
    artifact_name: &str,
) -> Result<Vec<std::path::PathBuf>, PlatformError> {
    let dir_entries = std::fs::read_dir(dir).map_err(|e| {
        PlatformError::internal(format!("Failed to read {} directory: {}", artifact_name, e))
    })?;

    let mut paths = Vec::new();
    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if is_loadable_yaml_entry(&path) {
            paths.push(path);
        }
    }

    Ok(paths)
}

fn is_loadable_yaml_entry(path: &Path) -> bool {
    path.is_file()
        && path.extension().and_then(|s| s.to_str()) == Some("yaml")
        && path.file_name().and_then(|s| s.to_str()) != Some("README.yaml")
}
