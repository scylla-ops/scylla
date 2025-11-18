//! Integration tests for ListJobsByStatusUseCase

use scylla_core::application::dto::ListJobsByStatusRequestDto;
use scylla_core::application::use_cases::job::list_jobs_by_status::ListJobsByStatusUseCase;
use scylla_core::domain::entities::{Job, Pipeline};
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::{JobStatus, PaginationParams, PipelineContent};
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
async fn test_list_jobs_by_status_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsByStatusUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("StatusPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create jobs with different statuses
    let job1 = create_test_job(created_pipeline.id(), "Job1");
    let job2 = create_test_job(created_pipeline.id(), "Job2");
    let job3 = create_test_job(created_pipeline.id(), "Job3");

    let _created1 = job_repo.create(&job1).await.unwrap();
    let created2 = job_repo.create(&job2).await.unwrap();
    let created3 = job_repo.create(&job3).await.unwrap();

    // Update statuses
    let mut job2_updated = created2;
    job2_updated.update_status(JobStatus::running()).unwrap();
    job_repo.update(&job2_updated).await.unwrap();

    let mut job3_updated = created3;
    job3_updated.update_status(JobStatus::running()).unwrap();
    job_repo.update(&job3_updated).await.unwrap();

    // List pending jobs
    let request = ListJobsByStatusRequestDto {
        status: JobStatus::pending(),
        pagination: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List jobs by status should succeed");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 1, "Should have 1 pending job");
}

#[tokio::test]
#[serial]
async fn test_list_jobs_by_status_use_case_running() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsByStatusUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("RunningPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create and update jobs to running
    for i in 0..3 {
        let job = create_test_job(created_pipeline.id(), &format!("RunningJob{}", i));
        let created = job_repo.create(&job).await.unwrap();
        let mut updated = created;
        updated.update_status(JobStatus::running()).unwrap();
        job_repo.update(&updated).await.unwrap();
    }

    let request = ListJobsByStatusRequestDto {
        status: JobStatus::running(),
        pagination: None,
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List running jobs should succeed");
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 3, "Should have 3 running jobs");
}

#[tokio::test]
#[serial]
async fn test_list_jobs_by_status_use_case_with_pagination() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
    let use_case = ListJobsByStatusUseCase::new(job_repo.clone());

    let pipeline = create_test_pipeline("PaginatedStatusPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create 10 completed jobs (must go through running first)
    for i in 0..10 {
        let job = create_test_job(created_pipeline.id(), &format!("CompletedJob{}", i));
        let created = job_repo.create(&job).await.unwrap();
        let mut running = created;
        running.update_status(JobStatus::running()).unwrap();
        job_repo.update(&running).await.unwrap();
        let mut completed = running;
        completed.update_status(JobStatus::completed()).unwrap();
        job_repo.update(&completed).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListJobsByStatusRequestDto {
        status: JobStatus::completed(),
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List jobs by status with pagination should succeed"
    );
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_jobs_by_status_use_case_empty() {
    let db = setup_test_db().await;
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db));
    let use_case = ListJobsByStatusUseCase::new(job_repo);

    let request = ListJobsByStatusRequestDto {
        status: JobStatus::failed(),
        pagination: None,
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List jobs by status should succeed even when empty"
    );
    let response = result.unwrap();
    assert_eq!(response.jobs.len(), 0);
}
