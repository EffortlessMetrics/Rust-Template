use std::str::FromStr;
use std::sync::Arc;

use tonic::{Request, Response, Status, transport::Server};
pub mod task {
    tonic::include_proto!("task");
}

use business_core::ports::TaskRepository;
use business_core::use_cases;
use model::{Task as ModelTask, TaskStatus};
use prost_types::Timestamp;
use task::{
    CreateTaskRequest, CreateTaskResponse, GetTaskRequest, ListTasksRequest, ListTasksResponse,
    Task as ProtoTask, UpdateTaskStatusRequest,
    task_service_server::{TaskService, TaskServiceServer},
};

pub struct TaskServiceImpl {
    repo: Arc<dyn TaskRepository>,
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let title = request.into_inner().title;
        let task = use_cases::create_task(&*self.repo, title).map_err(|e| Status::internal(e))?;
        let proto_task = model_task_to_proto(&task);
        Ok(Response::new(CreateTaskResponse { task: Some(proto_task) }))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<ProtoTask>, Status> {
        let id = request.into_inner().id.ok_or(Status::invalid_argument("id required"))?.value;
        let id_str = uuid::Uuid::from_bytes(
            id.as_slice().try_into().map_err(|_| Status::invalid_argument("invalid uuid"))?,
        )
        .to_string();
        let task =
            core::use_cases::get_task(&*self.repo, id_str).map_err(|e| Status::internal(e))?;
        let proto_task =
            task.map(model_task_to_proto).ok_or(Status::not_found("task not found"))?;
        Ok(Response::new(proto_task))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let tasks = core::use_cases::list_tasks(&*self.repo).map_err(|e| Status::internal(e))?;
        let proto_tasks = tasks.into_iter().map(model_task_to_proto).collect();
        Ok(Response::new(ListTasksResponse { tasks: proto_tasks }))
    }

    async fn update_status(
        &self,
        request: Request<UpdateTaskStatusRequest>,
    ) -> Result<Response<ProtoTask>, Status> {
        let inner = request.into_inner();
        let id = uuid::Uuid::from_bytes(
            inner
                .id
                .ok_or(Status::invalid_argument("id required"))?
                .value
                .as_slice()
                .try_into()
                .map_err(|_| Status::invalid_argument("invalid uuid"))?,
        )
        .to_string();
        let status_str = inner.status;
        let status = status_str
            .parse::<model::TaskStatus>()
            .map_err(|_| Status::invalid_argument("invalid status"))?;
        let task = core::use_cases::update_task_status(&*self.repo, id, status)
            .map_err(|e| Status::internal(e))?;
        let proto_task =
            task.map(model_task_to_proto).ok_or(Status::not_found("task not found"))?;
        Ok(Response::new(proto_task))
    }
}

fn model_task_to_proto(task: &ModelTask) -> ProtoTask {
    ProtoTask {
        id: Some(Uuid::from_str(&task.id).map_err(|_| "invalid uuid")?.into()),
        title: task.title.clone(),
        status: task.status.to_string(),
        created_at: Some(Timestamp::from(std::time::UNIX_EPOCH)),
    }
}

pub async fn spawn(
    port: u16,
    repo: Arc<dyn TaskRepository>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("[::1]:{}", port).parse()?;
    let service = TaskServiceImpl { repo };

    Server::builder().add_service(TaskServiceServer::new(service)).serve(addr).await?;

    Ok(())
}

#[cfg(feature = "integration-grpc")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    struct MockRepo {
        tasks: Arc<Mutex<HashMap<String, ModelTask>>>,
    }

    impl TaskRepository for MockRepo {
        fn save(&self, task: &ModelTask) -> Result<(), String> {
            let mut tasks = self.tasks.blocking_lock();
            tasks.insert(task.id.clone(), task.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_task_integration() {
        let tasks = Arc::new(Mutex::new(HashMap::new()));
        let repo = Arc::new(MockRepo { tasks });

        let service = TaskServiceImpl { repo: repo.clone() };
        // Start server in background
        let (tx, rx) = tokio::sync::oneshot::channel();
        let repo_clone = repo.clone();
        tokio::spawn(async move {
            let _ = spawn(0, repo_clone).await;
        });

        // Mock client - integration test needs real server spawn
        let response = CreateTaskResponse {
            task: Some(model_task_to_proto(&ModelTask {
                id: "test-id".to_string(),
                title: "Test task".to_string(),
                status: TaskStatus::Pending,
                created_at: chrono::Utc::now(),
            })),
        };
        assert_eq!(response.task.unwrap().title, "Test task");

        let request = tonic::Request::new(CreateTaskRequest { title: "Test task".to_string() });

        let response = client.create_task(request).await.unwrap().into_inner();
        assert_eq!(response.task.unwrap().title, "Test task");
    }
}
