//! Integration tests for GetJobUseCase

use scylla_core::application::dto::GetJobRequestDto;
use scylla_core::application::use_cases::job::get_job::GetJobUseCase;
use scylla_core::domain::entities::{Job, Pipeline};
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::job_repository::SurrealJobRepository;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

fn create_test_pipeline(content: &str) -> Pipeline {
    Pipeline::create(PipelineContent::new(content.to_string()).unwrap()).unwrap()
}

fn create_test_job(
    pipeline_id: &scylla_core::domain::value_objects::PipelineId,
    content: &str,
) -> Job {
    let pipeline_content = PipelineContent::new(content.to_string()).unwrap();
    Job::create(pipeline_id.clone(), pipeline_content).unwrap()
}

#[tokio::test]
#[serial]
async fn test_get_job_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = GetJobUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("GetPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "GetJob");
    let created = job_repo.create(&job).await.unwrap();

    let request = GetJobRequestDto {
        job_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Get job should succeed");
    let response = result.unwrap();
    assert_eq!(response.content.as_str(), "GetJob");
    assert_eq!(response.status.as_str(), "pending");
}

#[tokio::test]
#[serial]
async fn test_get_job_use_case_not_found() {
    let db = setup_test_db().await;
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db));
    let use_case = GetJobUseCase::new(job_repo);

    let request = GetJobRequestDto {
        job_id: scylla_core::domain::value_objects::JobId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Get non-existent job should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
