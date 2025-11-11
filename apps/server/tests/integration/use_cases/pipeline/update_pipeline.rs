//! Integration tests for UpdatePipelineUseCase

use scylla_core::application::dto::UpdatePipelineRequestDto;
use scylla_core::application::use_cases::pipeline::update_pipeline::UpdatePipelineUseCase;
use scylla_core::domain::entities::Pipeline;
use scylla_core::domain::repositories::PipelineRepository;
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

#[tokio::test]
#[serial]
async fn test_update_pipeline_use_case_update_content() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let use_case = UpdatePipelineUseCase::new(pipeline_repo.clone());

    let pipeline = Pipeline::create(PipelineContent::new("Original".to_string()).unwrap()).unwrap();
    let created = pipeline_repo.create(&pipeline).await.unwrap();

    let request = UpdatePipelineRequestDto {
        pipeline_id: created.id().clone(),
        content: PipelineContent::new("Updated content".to_string()).unwrap(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Update pipeline should succeed");
    let response = result.unwrap();
    assert_eq!(response.content.as_str(), "Updated content");

    // Verify it's persisted
    let updated = pipeline_repo.find_by_id(&response.id).await.unwrap();
    assert_eq!(updated.content().as_str(), "Updated content");
}

#[tokio::test]
#[serial]
async fn test_update_pipeline_use_case_not_found() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> = Arc::new(SurrealPipelineRepository::new(db));
    let use_case = UpdatePipelineUseCase::new(pipeline_repo);

    let request = UpdatePipelineRequestDto {
        pipeline_id: scylla_core::domain::value_objects::PipelineId::generate(),
        content: PipelineContent::new("New content".to_string()).unwrap(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Update non-existent pipeline should fail");
}
