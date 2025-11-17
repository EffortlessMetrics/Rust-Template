/// Commands module - placeholder for future command implementations
///
/// This module will contain shared command implementations that can be
/// used by xtask wrappers in brownfield and greenfield projects.
pub mod init {
    pub use crate::{init, InitMode};
}

pub mod selftest {
    pub use crate::selftest;
}
