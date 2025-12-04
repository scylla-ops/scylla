//! Integration tests for UpdateJobUseCase

use scylla_core::application::dto::UpdateJobRequestDto;
use scylla_core::application::use_cases::job::update_job::UpdateJobUseCase;
use scylla_core::domain::entities::{Job, Pipeline};
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::{JobStatus, PipelineContent};
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
async fn test_update_job_use_case_update_status() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = UpdateJobUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("UpdatePipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "Original");
    let created = job_repo.create(&job).await.unwrap();
    assert_eq!(created.status().as_str(), "pending");

    let request = UpdateJobRequestDto {
        job_id: created.id().clone(),
        status: Some(JobStatus::running()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update job should succeed");
    let response = result.unwrap();
    assert_eq!(response.status.as_str(), "running");

    // Verify it's persisted
    let updated = job_repo.find_by_id(&response.id).await.unwrap();
    assert_eq!(updated.status().as_str(), "running");
}

#[tokio::test]
#[serial]
async fn test_update_job_use_case_status_transitions() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = UpdateJobUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("StatusPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "StatusJob");
    let created = job_repo.create(&job).await.unwrap();

    // Test status transitions: pending -> running -> completed
    let request1 = UpdateJobRequestDto {
        job_id: created.id().clone(),
        status: Some(JobStatus::running()),
    };
    let result1 = use_case.execute(request1).await;
    assert!(result1.is_ok(), "Transition to running should succeed");

    let request2 = UpdateJobRequestDto {
        job_id: created.id().clone(),
        status: Some(JobStatus::completed()),
    };
    let result2 = use_case.execute(request2).await;
    assert!(result2.is_ok(), "Transition to completed should succeed");

    let final_job = job_repo.find_by_id(created.id()).await.unwrap();
    assert_eq!(final_job.status().as_str(), "completed");
}

#[tokio::test]
#[serial]
async fn test_update_job_use_case_not_found() {
    let db = setup_test_db().await;
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db));
    let use_case = UpdateJobUseCase::new(job_repo);

    let request = UpdateJobRequestDto {
        job_id: scylla_core::domain::value_objects::JobId::generate(),
        status: Some(JobStatus::running()),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Update non-existent job should fail");
}
