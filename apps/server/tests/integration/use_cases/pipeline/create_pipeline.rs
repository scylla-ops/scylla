//! Integration tests for CreatePipelineUseCase

use scylla_core::application::dto::CreatePipelineRequestDto;
use scylla_core::application::use_cases::pipeline::create_pipeline::CreatePipelineUseCase;
use scylla_core::domain::repositories::PipelineRepository;
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

#[tokio::test]
#[serial]
async fn test_create_pipeline_use_case_with_real_repository() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let use_case = CreatePipelineUseCase::new(pipeline_repo.clone());

    let request = CreatePipelineRequestDto {
        content: PipelineContent::new("test pipeline content".to_string()).unwrap(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Use case should succeed");
    let response = result.unwrap();
    assert_eq!(response.content.as_str(), "test pipeline content");

    // Verify pipeline exists in repository
    let pipeline = pipeline_repo
        .find_by_id(&response.id)
        .await
        .expect("Pipeline should exist");
    assert_eq!(pipeline.content().as_str(), "test pipeline content");
}
