//! Generic YAML resource repository.
//!
//! Provides standardized CRUD operations for governance artifacts stored as YAML files.

use crate::error::PlatformError;
use crate::pagination::{PaginatedResponse, Pagination, PaginationParams};
pub use gov_model::YamlResource;
use std::fs;
use std::path::{Path, PathBuf};

/// Generic repository for YAML-based resources.
pub struct YamlResourceRepo<T: YamlResource> {
    dir_path: PathBuf,
    max_files: usize,
    max_depth: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: YamlResource> YamlResourceRepo<T> {
    /// Create a new repository for the given directory.
    pub fn new(root: &Path, dir_name: &str) -> Self {
        Self {
            dir_path: root.join(dir_name),
            max_files: 1000,
            max_depth: 10,
            _phantom: std::marker::PhantomData,
        }
    }

    /// List all resources with pagination and optional filtering.
    pub fn list(
        &self,
        params: PaginationParams,
        filter: impl Fn(&T) -> bool,
        sort: impl Fn(&T, &T) -> std::cmp::Ordering,
    ) -> Result<PaginatedResponse<T>, PlatformError> {
        if !self.dir_path.exists() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                pagination: Pagination::new(0, params.page, params.per_page),
            });
        }

        let mut entries = Vec::new();
        let dir_entries = fs::read_dir(&self.dir_path).map_err(|e| {
            PlatformError::internal(format!(
                "Failed to read directory {}: {}",
                self.dir_path.display(),
                e
            ))
        })?;

        for (count, entry) in dir_entries.enumerate() {
            if count >= self.max_files {
                return Err(PlatformError::too_many_entries(count, self.max_files));
            }

            let entry = entry
                .map_err(|e| PlatformError::internal(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if !path.is_file()
                || path.extension().and_then(|s| s.to_str()) != Some("yaml")
                || path.file_name().and_then(|s| s.to_str()) == Some("README.yaml")
            {
                continue;
            }

            match self.load_from_path(&path) {
                Ok(resource) => {
                    if filter(&resource) {
                        entries.push(resource);
                    }
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = ?e, "Failed to load entry");
                }
            }
        }

        // Sort
        entries.sort_by(sort);

        let total = entries.len();
        let pagination = Pagination::new(total, params.page, params.per_page);

        let data = entries.into_iter().skip(pagination.offset()).take(pagination.limit()).collect();

        Ok(PaginatedResponse { data, pagination })
    }

    /// Load a resource by ID.
    pub fn get(&self, id: &str) -> Result<T, PlatformError> {
        let file_path = self.dir_path.join(format!("{}.yaml", id));

        if !file_path.exists() {
            return Err(PlatformError::not_found(format!("Resource '{}' not found", id)));
        }

        self.load_from_path(&file_path)
    }

    /// Save a resource.
    pub fn save(&self, resource: &T) -> Result<(), PlatformError> {
        if !self.dir_path.exists() {
            fs::create_dir_all(&self.dir_path).map_err(|e| {
                PlatformError::internal(format!("Failed to create directory: {}", e))
            })?;
        }

        let file_path = self.dir_path.join(format!("{}.yaml", resource.id()));
        let content = serde_yaml::to_string(resource)
            .map_err(|e| PlatformError::internal(format!("Failed to serialize resource: {}", e)))?;

        fs::write(&file_path, content).map_err(|e| {
            PlatformError::internal(format!("Failed to write file {}: {}", file_path.display(), e))
        })?;

        Ok(())
    }

    /// Delete a resource by ID.
    pub fn delete(&self, id: &str) -> Result<(), PlatformError> {
        let file_path = self.dir_path.join(format!("{}.yaml", id));

        if !file_path.exists() {
            return Err(PlatformError::not_found(format!("Resource '{}' not found", id)));
        }

        fs::remove_file(&file_path).map_err(|e| {
            PlatformError::internal(format!("Failed to delete file {}: {}", file_path.display(), e))
        })?;

        Ok(())
    }

    fn load_from_path(&self, path: &Path) -> Result<T, PlatformError> {
        let content = crate::safe_read_to_string(path).map_err(|e| {
            PlatformError::internal(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        let value: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
            PlatformError::internal(format!("Failed to parse YAML {}: {}", path.display(), e))
        })?;

        // Enforce depth limit
        spec_runtime::validate_yaml_depth(&value, self.max_depth).map_err(|e| {
            PlatformError::internal(format!("Invalid YAML depth in {}: {}", path.display(), e))
        })?;

        let resource: T = serde_yaml::from_value(value).map_err(|e| {
            PlatformError::internal(format!(
                "Failed to deserialize resource {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(resource)
    }
}
