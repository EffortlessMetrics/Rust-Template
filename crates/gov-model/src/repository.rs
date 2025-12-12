//! Governance repository trait for platform state access.

/// Repository trait for governance data access.
///
/// This is a minimal trait used by platform HTTP handlers for read-only
/// governance introspection. Concrete implementations live in business-core.
pub trait GovernanceRepository: Send + Sync {
    /// Get basic health status of the governance system.
    fn is_healthy(&self) -> bool;
}
