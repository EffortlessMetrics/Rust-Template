//! UI contract types.
use serde::{Deserialize, Serialize};

/// UI contract specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiContract {
    pub schema_version: String,
    pub template_version: String,
    pub screens: Vec<Screen>,
}

/// Screen definition with route and regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    pub id: String,
    pub route: String,
    pub description: String,
    pub regions: Vec<Region>,
}

/// UI region with stable data-uiid identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub kind: String,
    pub description: String,
}
