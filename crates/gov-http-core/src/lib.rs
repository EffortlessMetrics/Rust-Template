pub mod error;
pub mod extractors;
pub mod pagination;
pub mod repo;
pub mod state;

pub use error::{ErrorResponse, PlatformError};
pub use extractors::RequestId;
pub use gov_model::YamlResource;
pub use pagination::{PaginatedResponse, Pagination, PaginationParams};
pub use repo::YamlResourceRepo;
pub use state::PlatformState;

use axum::Router;

/// Maximum size for governed YAML files (10 MiB)
pub const MAX_YAML_SIZE: u64 = 10 * 1024 * 1024;

/// Read a file to string while preventing symlink traversal and size-based DoS.
pub fn safe_read_to_string(path: &std::path::Path) -> Result<String, std::io::Error> {
    let metadata = path.symlink_metadata()?;
    if metadata.is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("Symlinks not allowed: {}", path.display()),
        ));
    }

    if metadata.len() > MAX_YAML_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "File too large ({} bytes, max {} bytes): {}",
                metadata.len(),
                MAX_YAML_SIZE,
                path.display()
            ),
        ));
    }

    std::fs::read_to_string(path)
}

/// Build a router with health check endpoints.
pub fn router() -> Router {
    Router::new().route("/health", axum::routing::get(|| async { "OK" }))
}
