use crate::application::ports::PipelineRepository;
use crate::domain::entities::{OrganizationId, Pipeline, PipelineId, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;
use tracing::instrument;

pub struct SurrealPipelineRepository {
    db: Surreal<Any>,
}

impl SurrealPipelineRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PipelineRepository for SurrealPipelineRepository {
    #[instrument(skip(self, pipeline), fields(pipeline_id = %pipeline.id()))]
    async fn create(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        let db = self.db.clone();
        let pipeline = pipeline.clone();
        let created: Option<Pipeline> = db
            .create(RecordId::new(
                PipelineId::table_name(),
                pipeline.id().as_str(),
            ))
            .content(pipeline.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    #[instrument(skip(self), fields(pipeline_id = %id))]
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<Pipeline> = db
            .select(RecordId::new(PipelineId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        result.ok_or_else(|| DomainError::not_found("Pipeline", id.to_string()))
    }

    #[instrument(skip(self, pipeline), fields(pipeline_id = %pipeline.id()))]
    async fn update(&self, pipeline: &Pipeline) -> DomainResult<Pipeline> {
        let db = self.db.clone();
        let pipeline = pipeline.clone();
        let updated: Option<Pipeline> = db
            .update(RecordId::new(
                PipelineId::table_name(),
                pipeline.id().as_str(),
            ))
            .content(pipeline.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        updated.ok_or_else(|| DomainError::not_found("Pipeline", pipeline.id().to_string()))
    }

    #[instrument(skip(self), fields(pipeline_id = %id))]
    async fn delete(&self, id: &PipelineId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<Pipeline>>(RecordId::new(PipelineId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = PipelineId::table_name().to_string();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let pipelines: Vec<Pipeline> = db
            .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(pipelines, &params, total_count))
    }

    #[instrument(skip(self), fields(project_id = %project_id))]
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = PipelineId::table_name().to_string();
        let project_id = project_id.clone();

        let count_result: Vec<i64> = db
            .query(
                "SELECT count() FROM type::table($table) WHERE project_id = $project_id GROUP ALL",
            )
            .bind(("table", table.clone()))
            .bind(("project_id", project_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let pipelines: Vec<Pipeline> = db
            .query("SELECT * FROM type::table($table) WHERE project_id = $project_id ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("project_id", project_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(pipelines, &params, total_count))
    }

    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let organization_id = organization_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM pipelines WHERE project_id IN (SELECT VALUE id FROM projects WHERE organization_id = $org_id) GROUP ALL")
            .bind(("org_id", organization_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let pipelines: Vec<Pipeline> = db
            .query("SELECT * FROM pipelines WHERE project_id IN (SELECT VALUE id FROM projects WHERE organization_id = $org_id) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("org_id", organization_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(pipelines, &params, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{OrganizationId, PipelineNode, ProjectId};
    use crate::domain::value_objects::PaginationParams;
    use crate::domain::value_objects::pipeline::{NodeId, PipelineName};
    use crate::infrastructure::test_utils::init_db;

    async fn setup() -> Surreal<Any> {
        init_db(&[PipelineId::table_name(), ProjectId::table_name()]).await
    }

    fn test_project_id() -> ProjectId {
        ProjectId::generate()
    }

    fn node(id: &str, deps: &[&str]) -> PipelineNode {
        PipelineNode::new(
            NodeId::new(id).unwrap(),
            deps.iter().map(|d| NodeId::new(*d).unwrap()).collect(),
            "echo".into(),
            vec![],
        )
        .unwrap()
    }

    fn test_pipeline(project_id: ProjectId) -> Pipeline {
        Pipeline::create(
            PipelineName::new("test-pipeline").unwrap(),
            project_id,
            vec![node("a", &[]), node("b", &["a"])],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let project_id = test_project_id();
        let pipeline = test_pipeline(project_id.clone());
        let pipeline_id = pipeline.id().clone();

        let created = repo.create(&pipeline).await.expect("Failed to create");
        assert_eq!(created.id(), &pipeline_id);
        assert_eq!(created.name(), pipeline.name());
        assert_eq!(created.project_id(), &project_id);
        assert_eq!(created.nodes().len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let pipeline = test_pipeline(test_project_id());
        let pipeline_id = pipeline.id().clone();

        repo.create(&pipeline).await.expect("Failed to create");

        let found = repo.find_by_id(&pipeline_id).await.expect("Failed to find");
        assert_eq!(found.id(), &pipeline_id);
        assert_eq!(found.nodes().len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let fake_id = PipelineId::generate();
        assert!(repo.find_by_id(&fake_id).await.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let mut pipeline = test_pipeline(test_project_id());
        repo.create(&pipeline).await.expect("Failed to create");

        let new_name = PipelineName::new("updated-pipeline").unwrap();
        pipeline.update_name(new_name.clone()).unwrap();

        let updated = repo.update(&pipeline).await.expect("Failed to update");
        assert_eq!(updated.name(), &new_name);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let pipeline = test_pipeline(test_project_id());
        let pipeline_id = pipeline.id().clone();

        repo.create(&pipeline).await.expect("Failed to create");
        repo.delete(&pipeline_id).await.expect("Failed to delete");
        assert!(repo.find_by_id(&pipeline_id).await.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);
        let project_id = test_project_id();

        let p1 = test_pipeline(project_id.clone());
        let p2 = Pipeline::create(
            PipelineName::new("second").unwrap(),
            project_id,
            vec![node("x", &[])],
        )
        .unwrap();

        repo.create(&p1).await.unwrap();
        repo.create(&p2).await.unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert!(result.items().len() >= 2);
    }

    #[tokio::test]
    async fn test_list_by_project() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let project_a = test_project_id();
        let project_b = test_project_id();

        let p1 = test_pipeline(project_a.clone());
        let p2 = test_pipeline(project_b);

        repo.create(&p1).await.unwrap();
        repo.create(&p2).await.unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_by_project(&project_a, Some(&pagination))
            .await
            .unwrap();

        assert_eq!(result.items().len(), 1);
        assert_eq!(result.items()[0].project_id(), &project_a);
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_all_default_pagination() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let result = repo.list_all(None).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_by_project_empty() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let project_id = test_project_id();
        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_by_project(&project_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_by_organization_empty() {
        let db = setup().await;
        let repo = SurrealPipelineRepository::new(db);

        let org_id = OrganizationId::generate();
        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_by_organization(&org_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }
}
