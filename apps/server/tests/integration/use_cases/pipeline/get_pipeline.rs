//! Integration tests for GetPipelineUseCase

use scylla_core::application::dto::GetPipelineRequestDto;
use scylla_core::application::use_cases::pipeline::get_pipeline::GetPipelineUseCase;
use scylla_core::domain::entities::Pipeline;
use scylla_core::domain::repositories::PipelineRepository;
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

#[tokio::test]
#[serial]
async fn test_get_pipeline_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> = Arc::new(SurrealPipelineRepository::new(db));
    let use_case = GetPipelineUseCase::new(pipeline_repo.clone());

    let pipeline =
        Pipeline::create(PipelineContent::new("GetPipeline".to_string()).unwrap()).unwrap();
    let created = pipeline_repo.create(&pipeline).await.unwrap();

    let request = GetPipelineRequestDto {
        pipeline_id: created.id().clone(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "Get pipeline should succeed");
    let response = result.unwrap();
    assert_eq!(response.content.as_str(), "GetPipeline");
}

#[tokio::test]
#[serial]
async fn test_get_pipeline_use_case_not_found() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> = Arc::new(SurrealPipelineRepository::new(db));
    let use_case = GetPipelineUseCase::new(pipeline_repo);

    let request = GetPipelineRequestDto {
        pipeline_id: scylla_core::domain::value_objects::PipelineId::generate(),
    };

    let result = use_case.execute(request).await;

    assert!(result.is_err(), "Get non-existent pipeline should fail");
    match result.unwrap_err() {
        scylla_core::domain::errors::DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
