//! Integration tests for PipelineRepository
//!
//! These tests verify the PipelineRepository implementation against a real SurrealDB in-memory database.

use scylla_core::domain::entities::Pipeline;
use scylla_core::domain::errors::DomainError;
use scylla_core::domain::repositories::PipelineRepository;
use scylla_core::domain::value_objects::PipelineContent;
use scylla_core::infrastructure::persistence::surrealdb::pipeline_repository::SurrealPipelineRepository;
use serial_test::serial;

use crate::common::setup_test_db;

/// Helper to create a test pipeline entity
fn create_test_pipeline(content: &str) -> Pipeline {
    Pipeline::create(PipelineContent::new(content.to_string()).unwrap()).unwrap()
}

// ===== CREATE Tests =====

#[tokio::test]
#[serial]
async fn test_create_pipeline_success() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline = create_test_pipeline("test pipeline content");

    let created = repo
        .create(&pipeline)
        .await
        .expect("Creating pipeline should succeed");

    assert_eq!(created.content().as_str(), "test pipeline content");

    let retrieved = repo
        .find_by_id(created.id())
        .await
        .expect("Should be able to retrieve created pipeline");

    assert_eq!(retrieved.content().as_str(), "test pipeline content");
}

// ===== FIND_BY_ID Tests =====

#[tokio::test]
#[serial]
async fn test_find_by_id_success() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline = create_test_pipeline("FindMe");
    let created = repo.create(&pipeline).await.unwrap();

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Should find pipeline by ID");

    assert_eq!(found.content().as_str(), "FindMe");
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline_id = scylla_core::domain::value_objects::PipelineId::generate();

    let result = repo.find_by_id(&pipeline_id).await;

    assert!(result.is_err(), "Should return error when not found");
    match result.unwrap_err() {
        DomainError::NotFound { .. } => {}
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

// ===== UPDATE Tests =====

#[tokio::test]
#[serial]
async fn test_update_pipeline_success() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline = create_test_pipeline("Original");
    let created = repo.create(&pipeline).await.unwrap();

    let mut updated = created;
    let new_content = PipelineContent::new("Updated content".to_string()).unwrap();
    updated.update_content(new_content).unwrap();

    let persisted = repo.update(&updated).await.expect("Update should succeed");

    assert_eq!(persisted.content().as_str(), "Updated content");
}

// ===== DELETE Tests =====

#[tokio::test]
#[serial]
async fn test_delete_pipeline_success() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline = create_test_pipeline("DeleteMe");
    let created = repo.create(&pipeline).await.unwrap();

    repo.delete(created.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(created.id()).await;
    assert!(result.is_err(), "Pipeline should be deleted");
}

#[tokio::test]
#[serial]
async fn test_delete_nonexistent_pipeline() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline_id = scylla_core::domain::value_objects::PipelineId::generate();

    let result = repo.delete(&pipeline_id).await;
    assert!(
        result.is_ok(),
        "Deleting non-existent pipeline should not error"
    );
}

// ===== LIST_ALL Tests =====

#[tokio::test]
#[serial]
async fn test_list_all_pipelines() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    for i in 0..5 {
        let pipeline = create_test_pipeline(&format!("Pipeline{}", i));
        repo.create(&pipeline).await.unwrap();
    }

    let result = repo.list_all(None).await.expect("Listing should succeed");

    assert_eq!(result.items().len(), 5);
    assert_eq!(result.metadata().total_count(), 5);
}

// ===== Complex Scenarios =====

#[tokio::test]
#[serial]
async fn test_pipeline_full_lifecycle() {
    let db = setup_test_db().await;
    let repo = SurrealPipelineRepository::new(db);

    let pipeline = create_test_pipeline("Lifecycle");
    let created = repo.create(&pipeline).await.expect("Create should succeed");

    let found = repo
        .find_by_id(created.id())
        .await
        .expect("Find should succeed");
    assert_eq!(found.content().as_str(), "Lifecycle");

    let mut updated = found;
    updated
        .update_content(PipelineContent::new("UpdatedLifecycle".to_string()).unwrap())
        .unwrap();
    let persisted = repo.update(&updated).await.expect("Update should succeed");
    assert_eq!(persisted.content().as_str(), "UpdatedLifecycle");

    repo.delete(persisted.id())
        .await
        .expect("Delete should succeed");

    let result = repo.find_by_id(persisted.id()).await;
    assert!(result.is_err(), "Pipeline should be deleted");
}
