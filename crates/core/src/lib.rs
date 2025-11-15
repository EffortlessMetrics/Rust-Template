// Core business logic goes here
//
// This crate should contain:
// - Domain entities and business rules
// - Use case / application service logic
// - Port definitions (traits for adapters to implement)
//
// Architecture principles:
// - No dependencies on HTTP, database, or other adapters
// - Adapters (app-http, app-db, etc.) call core, never the reverse
// - Core defines ports (traits), adapters implement them
//
// Example structure:
//
// pub mod entities {
//     pub struct Task { pub id: String, pub title: String }
// }
//
// pub mod ports {
//     pub trait TaskRepository {
//         async fn save(&self, task: &Task) -> Result<(), Error>;
//     }
// }
//
// pub mod use_cases {
//     pub async fn create_task(repo: &impl TaskRepository, title: String) -> Result<Task, Error> {
//         let task = Task { id: uuid::new_v4(), title };
//         repo.save(&task).await?;
//         Ok(task)
//     }
// }
//
// See docs/tutorials/first-ac-change.md for guidance on adding your first domain feature.
