use std::path::Path;

use gov_http_core::PlatformError;
use gov_http_types::{FrictionEntry, Question};

pub(crate) fn load_friction_entries(root: &Path) -> Result<Vec<FrictionEntry>, PlatformError> {
    load_yaml_entries(root, "friction", "friction", load_friction_entry)
}

fn load_friction_entry(path: &Path) -> Result<FrictionEntry, PlatformError> {
    load_yaml_entry(path, "friction")
}

pub(crate) fn load_question_entries(root: &Path) -> Result<Vec<Question>, PlatformError> {
    load_yaml_entries(root, "questions", "question", load_question_entry)
}

fn load_question_entry(path: &Path) -> Result<Question, PlatformError> {
    load_yaml_entry(path, "question")
}

fn load_yaml_entries<T, F>(
    root: &Path,
    directory_name: &str,
    entry_name: &str,
    load_entry: F,
) -> Result<Vec<T>, PlatformError>
where
    F: Fn(&Path) -> Result<T, PlatformError>,
{
    let directory = root.join(directory_name);

    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let dir_entries = std::fs::read_dir(&directory).map_err(|e| {
        PlatformError::internal(format!("Failed to read {} directory: {}", directory_name, e))
    })?;

    for entry in dir_entries {
        let entry = entry.map_err(|e| {
            PlatformError::internal(format!("Failed to read directory entry: {}", e))
        })?;
        let path = entry.path();

        if !is_loadable_yaml_file(&path) {
            continue;
        }

        match load_entry(&path) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = ?e,
                    "Failed to load {} entry",
                    entry_name
                );
            }
        }
    }

    Ok(entries)
}

fn is_loadable_yaml_file(path: &Path) -> bool {
    path.is_file()
        && path.extension().and_then(|s| s.to_str()) == Some("yaml")
        && path.file_name().and_then(|s| s.to_str()) != Some("README.yaml")
}

fn load_yaml_entry<T>(path: &Path, entry_name: &str) -> Result<T, PlatformError>
where
    T: serde::de::DeserializeOwned,
{
    let content = std::fs::read_to_string(path).map_err(|e| {
        PlatformError::internal(format!(
            "Failed to read {} file {}: {}",
            entry_name,
            path.display(),
            e
        ))
    })?;

    serde_yaml::from_str(&content).map_err(|e| {
        PlatformError::internal(format!(
            "Failed to parse {} YAML {}: {}",
            entry_name,
            path.display(),
            e
        ))
    })
}
