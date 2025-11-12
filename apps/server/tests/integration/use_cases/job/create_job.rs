//! Integration tests for CreateJobUseCase

use scylla_core::application::dto::CreateJobRequestDto;
use scylla_core::application::use_cases::job::create_job::CreateJobUseCase;
use scylla_core::domain::entities::Pipeline;
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::job_repository::SurrealJobRepository;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

#[tokio::test]
#[serial]
async fn test_create_job_use_case_with_real_repository() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = CreateJobUseCase::new(job_repo.clone(), pipeline_repo.clone());

    // Create a pipeline first
    let pipeline =
        Pipeline::create(PipelineContent::new("pipeline content".to_string()).unwrap()).unwrap();
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let request = CreateJobRequestDto {
        pipeline_id: created_pipeline.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Use case should succeed");
    let response = result.unwrap();
    assert_eq!(response.status.as_str(), "pending");
    assert_eq!(
        response.pipeline_id.as_str(),
        created_pipeline.id().as_str()
    );

    // Verify job exists in repository
    let job = job_repo
        .find_by_id(&response.id)
        .await
        .expect("Job should exist");
    assert_eq!(job.status().as_str(), "pending");
}

#[tokio::test]
#[serial]
async fn test_create_job_use_case_pipeline_not_found() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = CreateJobUseCase::new(job_repo, pipeline_repo);

    let request = CreateJobRequestDto {
        pipeline_id: scylla_core::domain::value_objects::PipelineId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_err(),
        "Creating job with non-existent pipeline should fail"
    );
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
