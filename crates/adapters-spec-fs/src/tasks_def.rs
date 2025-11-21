use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TasksFile {
    pub schema_version: String,
    pub template_version: String,
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskDefinition {
    pub id: String,
    pub title: String,
    pub requirement: String,
    #[serde(default)]
    pub acs: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub recommended_flows: Vec<String>,
    #[serde(default)]
    pub docs: TaskDocs,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TaskDocs {
    #[serde(default)]
    pub design: Vec<String>,
    #[serde(default)]
    pub plan: Vec<String>,
}

pub fn load_tasks_definitions(
    path: &std::path::Path,
) -> Result<HashMap<String, TaskDefinition>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read tasks.yaml: {}", e))?;

    let tasks_file: TasksFile =
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse tasks.yaml: {}", e))?;

    let mut map = HashMap::new();
    for task in tasks_file.tasks {
        map.insert(task.id.clone(), task);
    }

    Ok(map)
}
