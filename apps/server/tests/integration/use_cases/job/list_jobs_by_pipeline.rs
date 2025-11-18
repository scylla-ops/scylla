//! Integration tests for ListJobsByPipelineUseCase

use scylla_core::application::dto::ListJobsByPipelineRequestDto;
use scylla_core::application::use_cases::job::list_jobs_by_pipeline::ListJobsByPipelineUseCase;
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
async fn test_list_jobs_by_pipeline_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsByPipelineUseCase::new(job_repo.clone());

    let pipeline1 = create_test_pipeline("Pipeline1");
    let pipeline2 = create_test_pipeline("Pipeline2");
    let created_pipeline1 = pipeline_repo.create(&pipeline1).await.unwrap();
    let created_pipeline2 = pipeline_repo.create(&pipeline2).await.unwrap();

    // Create 3 jobs for pipeline1
    for i in 0..3 {
        let job = create_test_job(created_pipeline1.id(), &format!("Job1_{}", i));
        job_repo.create(&job).await.unwrap();
    }

    // Create 2 jobs for pipeline2
    for i in 0..2 {
        let job = create_test_job(created_pipeline2.id(), &format!("Job2_{}", i));
        job_repo.create(&job).await.unwrap();
    }

    let request = ListJobsByPipelineRequestDto {
        pipeline_id: created_pipeline1.id().clone(),
        pagination: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List jobs by pipeline should succeed");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 3, "Should have 3 jobs for pipeline1");
}

#[tokio::test]
#[serial]
async fn test_list_jobs_by_pipeline_use_case_with_pagination() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsByPipelineUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("PaginatedPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create 10 jobs
    for i in 0..10 {
        let job = create_test_job(created_pipeline.id(), &format!("PaginatedJob{}", i));
        job_repo.create(&job).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListJobsByPipelineRequestDto {
        pipeline_id: created_pipeline.id().clone(),
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List jobs by pipeline with pagination should succeed"
    );
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_list_jobs_by_pipeline_use_case_empty() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db));
    let use_case = ListJobsByPipelineUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("EmptyPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let request = ListJobsByPipelineRequestDto {
        pipeline_id: created_pipeline.id().clone(),
        pagination: None,
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List jobs by pipeline should succeed even when empty"
    );
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 0);
}
