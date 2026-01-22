//! End-to-end test for pipeline creation workflow

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
async fn test_create_pipeline_end_to_end() {
    let db = setup_test_db().await;

    // Setup repositories
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));

    // Setup use case
    let create_pipeline_use_case = CreatePipelineUseCase::new(pipeline_repo.clone());

    // Execute the use case
    let request = CreatePipelineRequestDto {
        content: PipelineContent::new("test pipeline content".to_string(), Default::default())
            .unwrap(),
    };

    let result = create_pipeline_use_case.execute(request).await;

    assert!(result.is_ok(), "Pipeline creation should succeed");
    let response = result.unwrap();

    // Verify the pipeline exists in the repository
    let pipeline = pipeline_repo
        .find_by_id(&response.id)
        .await
        .expect("Pipeline should exist in repository");

    assert_eq!(pipeline.content().as_str(), "test pipeline content");
    assert_eq!(pipeline.id().as_str(), response.id.as_str());
}
