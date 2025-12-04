//! End-to-end test for job creation workflow

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
async fn test_create_job_end_to_end() {
    let db = setup_test_db().await;

    // Setup repositories
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));

    // Create a pipeline first
    let pipeline =
        Pipeline::create(PipelineContent::new("pipeline content".to_string()).unwrap()).unwrap();
    let created_pipeline = pipeline_repo.create(&pipeline).await.unwrap();

    // Setup use case
    let create_job_use_case = CreateJobUseCase::new(job_repo.clone(), pipeline_repo.clone());

    // Execute the use case
    let request = CreateJobRequestDto {
        pipeline_id: created_pipeline.id().clone(),
    };

    let result = create_job_use_case.execute(request).await;

    assert!(result.is_ok(), "Job creation should succeed");
    let response = result.unwrap();

    // Verify the job exists in the repository
    let job = job_repo
        .find_by_id(&response.id)
        .await
        .expect("Job should exist in repository");

    assert_eq!(job.status().as_str(), "pending");
    assert_eq!(job.pipeline_id().as_str(), created_pipeline.id().as_str());
    assert_eq!(job.id().as_str(), response.id.as_str());
}
