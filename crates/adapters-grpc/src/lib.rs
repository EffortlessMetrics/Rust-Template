use std::sync::Arc;

use tonic::{Request, Response, Status, transport::Server};
pub mod task {
    pub mod v1 {
        tonic::include_proto!("task.v1");
    }
}

use business_core::ports::TaskRepository;
use business_core::use_cases;
use model::{Task as ModelTask, TaskStatus};
use prost_types::Timestamp;
use task::v1::{
    CreateTaskRequest, CreateTaskResponse, GetTaskRequest, GetTaskResponse, ListTasksRequest,
    ListTasksResponse, Task as ProtoTask, UpdateStatusRequest, UpdateStatusResponse,
    task_service_server::{TaskService, TaskServiceServer},
};

pub struct TaskServiceImpl {
    repo: Arc<dyn TaskRepository>,
}

impl TaskServiceImpl {
    /// Create a new TaskServiceImpl with the given repository (useful for testing)
    pub fn new(repo: Arc<dyn TaskRepository>) -> Self {
        Self { repo }
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        let title = request.into_inner().title;
        let task = use_cases::create_task(&*self.repo, title).await.map_err(Status::internal)?;
        let proto_task = model_task_to_proto(&task);
        Ok(Response::new(CreateTaskResponse { task: Some(proto_task) }))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        let id = request.into_inner().id;
        let task = use_cases::get_task(&*self.repo, id).await.map_err(Status::internal)?;
        let proto_task =
            task.map(|t| model_task_to_proto(&t)).ok_or(Status::not_found("task not found"))?;
        Ok(Response::new(GetTaskResponse { task: Some(proto_task) }))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let tasks = use_cases::list_tasks(&*self.repo).await.map_err(Status::internal)?;
        let proto_tasks = tasks.into_iter().map(|t| model_task_to_proto(&t)).collect();
        Ok(Response::new(ListTasksResponse { tasks: proto_tasks }))
    }

    async fn update_status(
        &self,
        request: Request<UpdateStatusRequest>,
    ) -> Result<Response<UpdateStatusResponse>, Status> {
        let inner = request.into_inner();
        let id = inner.id;
        let status = parse_task_status(&inner.status)?;
        let task = use_cases::update_task_status(&*self.repo, id, status)
            .await
            .map_err(Status::internal)?;
        let proto_task =
            task.map(|t| model_task_to_proto(&t)).ok_or(Status::not_found("task not found"))?;
        Ok(Response::new(UpdateStatusResponse { task: Some(proto_task) }))
    }
}

fn model_task_to_proto(task: &ModelTask) -> ProtoTask {
    ProtoTask {
        id: task.id.clone(),
        title: task.title.clone(),
        status: format!("{:?}", task.status),
        created_at: Some(Timestamp {
            seconds: task.created_at.timestamp(),
            nanos: task.created_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn parse_task_status(s: &str) -> Result<TaskStatus, Status> {
    match s {
        "Pending" => Ok(TaskStatus::Pending),
        "InProgress" => Ok(TaskStatus::InProgress),
        "Completed" => Ok(TaskStatus::Completed),
        _ => Err(Status::invalid_argument(format!("Invalid task status: {}", s))),
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
