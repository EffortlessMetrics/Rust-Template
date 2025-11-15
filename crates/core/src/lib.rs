use std::sync::atomic::{AtomicBool, Ordering};

// Test-only flag to simulate service unavailability for acceptance tests
// In production, this would be replaced with actual service health checks
static SERVICE_AVAILABLE: AtomicBool = AtomicBool::new(true);

pub fn refund_ok() -> bool {
    SERVICE_AVAILABLE.load(Ordering::Relaxed)
}

/// Set service availability (for testing and demos)
///
/// In a real production system, you would:
/// - Remove this function entirely
/// - Use actual health checks and circuit breakers
/// - Check database/external service connectivity
pub fn set_service_available(available: bool) {
    SERVICE_AVAILABLE.store(available, Ordering::Relaxed);
}
