//! Integration tests for ListJobsUseCase

use scylla_core::application::dto::ListJobsRequestDto;
use scylla_core::application::use_cases::job::list_jobs::ListJobsUseCase;
use scylla_core::domain::entities::{Job, Pipeline};
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::{PaginationParams, PipelineContent};
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
async fn test_list_jobs_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("ListPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create multiple jobs
    for i in 0..5 {
        let job = create_test_job(created_pipeline.id(), &format!("Job{}", i));
        job_repo.create(&job).await.unwrap();
    }

    let request = ListJobsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List jobs should succeed");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 5);
    assert!(response.pagination.is_some());
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 5);
    }
}

#[tokio::test]
#[serial]
async fn test_list_jobs_use_case_with_pagination() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("PaginatedPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create 10 jobs
    for i in 0..10 {
        let job = create_test_job(created_pipeline.id(), &format!("PaginatedJob{}", i));
        job_repo.create(&job).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListJobsRequestDto {
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List jobs with pagination should succeed");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_jobs_use_case_empty() {
    let db = setup_test_db().await;
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db));
    let use_case = ListJobsUseCase::new(job_repo);

    let request = ListJobsRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List jobs should succeed even when empty");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 0);
}
