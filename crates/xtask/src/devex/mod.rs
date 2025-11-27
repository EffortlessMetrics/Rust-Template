pub mod spec;

pub use spec::{DevExSpec, load_spec};

// Re-export CommandSpec for test code only
#[cfg(test)]
pub use spec::CommandSpec;
