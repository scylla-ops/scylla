//! Integration tests for ListPipelinesUseCase

use scylla_core::application::dto::ListPipelinesRequestDto;
use scylla_core::application::use_cases::pipeline::list_pipelines::ListPipelinesUseCase;
use scylla_core::domain::entities::Pipeline;
use scylla_core::domain::repositories::PipelineRepository;
use scylla_core::domain::value_objects::{PaginationParams, PipelineContent};
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;
use std::sync::Arc;

use crate::common::setup_test_db;

#[tokio::test]
#[serial]
async fn test_list_pipelines_use_case_success() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let use_case = ListPipelinesUseCase::new(pipeline_repo.clone());

    // Create multiple pipelines
    for i in 0..5 {
        let pipeline =
            Pipeline::create(PipelineContent::new(format!("Pipeline{}", i)).unwrap()).unwrap();
        pipeline_repo.create(&pipeline).await.unwrap();
    }

    let request = ListPipelinesRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(result.is_ok(), "List pipelines should succeed");
    let response = result.unwrap();
    assert_eq!(response.pipelines.len(), 5);
    assert!(response.pagination.is_some());
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 5);
    }
}

#[tokio::test]
#[serial]
async fn test_list_pipelines_use_case_with_pagination() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> =
        Arc::new(SurrealPipelineRepository::new(db.clone()));
    let use_case = ListPipelinesUseCase::new(pipeline_repo.clone());

    // Create 10 pipelines
    for i in 0..10 {
        let pipeline =
            Pipeline::create(PipelineContent::new(format!("PaginatedPipeline{}", i)).unwrap())
                .unwrap();
        pipeline_repo.create(&pipeline).await.unwrap();
    }

    let pagination = PaginationParams::new(1, 3).unwrap();
    let request = ListPipelinesRequestDto {
        pagination: Some(pagination),
    };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List pipelines with pagination should succeed"
    );
    let response = result.unwrap();
    assert_eq!(response.pipelines.len(), 3);
    if let Some(pagination) = response.pagination {
        assert_eq!(pagination.total_count(), 10);
    }
}

#[tokio::test]
#[serial]
async fn test_list_pipelines_use_case_empty() {
    let db = setup_test_db().await;
    let pipeline_repo: Arc<dyn PipelineRepository> = Arc::new(SurrealPipelineRepository::new(db));
    let use_case = ListPipelinesUseCase::new(pipeline_repo);

    let request = ListPipelinesRequestDto { pagination: None };

    let result = use_case.execute(request).await;

    assert!(
        result.is_ok(),
        "List pipelines should succeed even when empty"
    );
    let response = result.unwrap();
    assert_eq!(response.pipelines.len(), 0);
}
