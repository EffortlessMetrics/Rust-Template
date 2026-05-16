//! UI routes for platform visualization.
//!
//! Provides HTML-based UI for:
//! - Dashboard
//! - Graph visualization
//! - Flows and tasks
//! - AC coverage

mod coverage;
mod dashboard;
mod flows;
mod graph;
mod layout;

pub use coverage::coverage_view;
pub use dashboard::dashboard;
pub use flows::flows_view;
pub use graph::graph_view;
