//! Integration tests for JobRepository
//!
//! These tests verify the JobRepository implementation against a real SurrealDB in-memory database.

use scylla_core::domain::entities::{Job, Pipeline};
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::repositories::{JobRepository, PipelineRepository};
use scylla_core::domain::value_objects::{JobStatus, PipelineContent};
use scylla_core::infrastructure::persistence::surrealdb::job_repository::SurrealJobRepository;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;

use crate::common::setup_test_db;

/// Helper to create a test pipeline entity
fn create_test_pipeline(content: &str) -> Pipeline {
    Pipeline::create(PipelineContent::new(content.to_string()).unwrap()).unwrap()
}

/// Helper to create a test job entity
fn create_test_job(
    pipeline_id: &scylla_core::domain::value_objects::PipelineId,
    content: &str,
) -> Job {
    let pipeline_content = PipelineContent::new(content.to_string()).unwrap();
    Job::create(pipeline_id.clone(), pipeline_content).unwrap()
}

// ===== CREATE Tests =====

#[tokio::test]
#[serial]
async fn test_create_job_success() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("pipeline content");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "job content");

    let created = repo
        .create(&job)
        .await
        .expect("Creating job should succeed");

    assert_eq!(created.content().as_str(), "job content");
    assert_eq!(created.status().as_str(), "pending");
    assert_eq!(
        created.pipeline_id().as_str(),
        created_pipeline.id().as_str()
    );

    let retrieved = repo
        .find_by_id(created.id())
        .await
        .expect("Should be able to retrieve created job");

    assert_eq!(retrieved.content().as_str(), "job content");
}

// ===== FIND_BY_ID Tests =====

#[tokio::test]
#[serial]
async fn test_find_by_id_success() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("FindPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "FindMe");
    let created = repo.create(&job).await.unwrap();

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Should find job by ID");

    assert_eq!(found.content().as_str(), "FindMe");
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found() {
    let db = setup_test_db().await;
    let repo = SurrealJobRepository::new(db);

    let job_id = scylla_core::domain::value_objects::JobId::generate();

    let result = repo.find_by_id(&job_id).await;

    assert!(result.is_err(), "Should return error when not found");
    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== UPDATE Tests =====

#[tokio::test]
#[serial]
async fn test_update_job_status_success() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("UpdatePipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "Original");
    let created = repo.create(&job).await.unwrap();
    assert_eq!(created.status().as_str(), "pending");

    let mut updated = created;
    let new_status = JobStatus::new("running").unwrap();
    updated.update_status(new_status).unwrap();

    let persisted = repo.update(&updated).await.expect("Update should succeed");

    assert_eq!(persisted.status().as_str(), "running");
}

// ===== DELETE Tests =====

#[tokio::test]
#[serial]
async fn test_delete_job_success() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("DeletePipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "DeleteMe");
    let created = repo.create(&job).await.unwrap();

    repo.delete(created.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(created.id()).await;
    assert!(result.is_err(), "Job should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_nonexistent_job() {
    let db = setup_test_db().await;
    let repo = SurrealJobRepository::new(db);

    let job_id = scylla_core::domain::value_objects::JobId::generate();

    let result = repo.delete(&job_id).await;
    assert!(result.is_ok(), "Deleting non-existent job should not error");
}

// ===== LIST_ALL Tests =====

#[tokio::test]
#[serial]
async fn test_list_all_jobs() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("ListPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    for i in 0..5 {
        let job = create_test_job(created_pipeline.id(), &format!("Job{}", i));
        repo.create(&job).await.unwrap();
    }

    let result = repo.list_all(None).await.expect("Listing should succeed");

    assert_eq!(result.items().len(), 5);
    assert_eq!(result.metadata().total_count(), 5);
}

// ===== LIST_BY_PIPELINE Tests =====

#[tokio::test]
#[serial]
async fn test_list_jobs_by_pipeline() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline1 = create_test_pipeline("Pipeline1");
    let pipeline2 = create_test_pipeline("Pipeline2");
    let created_pipeline1 = pipeline_repo.create(&pipeline1).await.unwrap();
    let created_pipeline2 = pipeline_repo.create(&pipeline2).await.unwrap();

    // Create 3 jobs for pipeline1
    for i in 0..3 {
        let job = create_test_job(created_pipeline1.id(), &format!("Job1_{}", i));
        repo.create(&job).await.unwrap();
    }

    // Create 2 jobs for pipeline2
    for i in 0..2 {
        let job = create_test_job(created_pipeline2.id(), &format!("Job2_{}", i));
        repo.create(&job).await.unwrap();
    }

    let result = repo
        .list_by_pipeline(created_pipeline1.id(), None)
        .await
        .expect("Listing by pipeline should succeed");

    assert_eq!(result.items().len(), 3, "Should have 3 jobs for pipeline1");
}

// ===== LIST_BY_STATUS Tests =====

#[tokio::test]
#[serial]
async fn test_list_jobs_by_status() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("StatusPipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Create jobs with different statuses
    let job1 = create_test_job(created_pipeline.id(), "Job1");
    let job2 = create_test_job(created_pipeline.id(), "Job2");
    let job3 = create_test_job(created_pipeline.id(), "Job3");

    let _created1 = repo.create(&job1).await.unwrap();
    let created2 = repo.create(&job2).await.unwrap();
    let created3 = repo.create(&job3).await.unwrap();

    // Update statuses
    let mut job2_updated = created2;
    job2_updated.update_status(JobStatus::running()).unwrap();
    repo.update(&job2_updated).await.unwrap();

    let mut job3_updated = created3;
    job3_updated.update_status(JobStatus::running()).unwrap();
    repo.update(&job3_updated).await.unwrap();

    // List pending jobs
    let pending_result = repo
        .list_by_status(&JobStatus::pending(), None)
        .await
        .expect("Listing by status should succeed");

    assert_eq!(pending_result.items().len(), 1, "Should have 1 pending job");

    // List running jobs
    let running_result = repo
        .list_by_status(&JobStatus::running(), None)
        .await
        .expect("Listing by status should succeed");

    assert_eq!(
        running_result.items().len(),
        2,
        "Should have 2 running jobs"
    );
}

// ===== Complex Scenarios =====

#[tokio::test]
#[serial]
async fn test_job_full_lifecycle() {
    let db = setup_test_db().await;
    let pipeline_repo = SurrealPipelineRepository::new(db.clone());
    let repo = SurrealJobRepository::new(db);

    let pipeline = create_test_pipeline("LifecyclePipeline");
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    let job = create_test_job(created_pipeline.id(), "Lifecycle");
    let created = repo.create(&job).await.expect("Create should succeed");
    assert_eq!(created.status().as_str(), "pending");

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Find should succeed");
    assert_eq!(found.content().as_str(), "Lifecycle");

    let mut updated = found;
    updated.update_status(JobStatus::running()).unwrap();
    let persisted = repo.update(&updated).await.expect("Update should succeed");
    assert_eq!(persisted.status().as_str(), "running");

    // Now transition to completed
    let mut updated2 = persisted;
    updated2.update_status(JobStatus::completed()).unwrap();
    let persisted2 = repo.update(&updated2).await.expect("Update should succeed");
    assert_eq!(persisted2.status().as_str(), "completed");

    repo.delete(persisted2.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(persisted2.id()).await;
    assert!(result.is_err(), "Job should be deleted");
}
