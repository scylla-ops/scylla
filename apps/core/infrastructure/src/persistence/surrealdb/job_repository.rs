use async_trait::async_trait;
use domain::entities::{Job, JobId, OrganizationId, PipelineId, ProjectId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::JobRepository;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

pub struct SurrealJobRepository {
    db: Surreal<Any>,
}

impl SurrealJobRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobRepository for SurrealJobRepository {
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        let db = self.db.clone();
        let job = job.clone();
        let created: Option<Job> = db
            .create(RecordId::new(JobId::table_name(), job.id().as_str()))
            .content(job.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<Job> = db
            .select(RecordId::new(JobId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("Job", id.to_string()))
    }

    async fn update(&self, job: &Job) -> DomainResult<Job> {
        let db = self.db.clone();
        let job = job.clone();
        let updated: Option<Job> = db
            .update(RecordId::new(JobId::table_name(), job.id().as_str()))
            .content(job.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("Job", job.id().to_string()))
    }

    async fn delete(&self, id: &JobId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<Job>>(RecordId::new(JobId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = JobId::table_name().to_string();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let jobs: Vec<Job> = db
            .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(jobs, &params, total_count))
    }

    async fn list_by_pipeline(
        &self,
        pipeline_id: &PipelineId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = JobId::table_name().to_string();
        let pipeline_id = pipeline_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE pipeline_id = $pipeline_id GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("pipeline_id", pipeline_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let jobs: Vec<Job> = db
            .query("SELECT * FROM type::table($table) WHERE pipeline_id = $pipeline_id ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("pipeline_id", pipeline_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(jobs, &params, total_count))
    }

    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let project_id = project_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM jobs WHERE pipeline_id IN (SELECT VALUE id FROM pipelines WHERE project_id = $project_id) GROUP ALL")
            .bind(("project_id", project_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let jobs: Vec<Job> = db
            .query("SELECT * FROM jobs WHERE pipeline_id IN (SELECT VALUE id FROM pipelines WHERE project_id = $project_id) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("project_id", project_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(jobs, &params, total_count))
    }

    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let organization_id = organization_id.clone();

        let count_result: Vec<i64> = db
            .query("SELECT count() FROM jobs WHERE pipeline_id IN (SELECT VALUE id FROM pipelines WHERE project_id IN (SELECT VALUE id FROM projects WHERE organization_id = $org_id)) GROUP ALL")
            .bind(("org_id", organization_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let jobs: Vec<Job> = db
            .query("SELECT * FROM jobs WHERE pipeline_id IN (SELECT VALUE id FROM pipelines WHERE project_id IN (SELECT VALUE id FROM projects WHERE organization_id = $org_id)) ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("org_id", organization_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(jobs, &params, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_db;
    use domain::entities::{Pipeline, PipelineNode, ProjectId};
    use domain::value_objects::PaginationParams;
    use domain::value_objects::pipeline::{NodeId, PipelineName};

    async fn setup() -> Surreal<Any> {
        init_db(&[
            JobId::table_name(),
            PipelineId::table_name(),
            ProjectId::table_name(),
        ])
        .await
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

    fn test_pipeline() -> Pipeline {
        Pipeline::create(
            PipelineName::new("test").unwrap(),
            ProjectId::generate(),
            vec![node("a", &[]), node("b", &["a"])],
        )
        .unwrap()
    }

    fn test_job(pipeline: &Pipeline) -> Job {
        Job::create_from_pipeline(pipeline)
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline = test_pipeline();
        let job = test_job(&pipeline);
        let job_id = job.id().clone();

        let created = repo.create(&job).await.expect("Failed to create");
        assert_eq!(created.id(), &job_id);
        assert_eq!(created.pipeline_id(), pipeline.id());
        assert_eq!(created.node_executions().len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline = test_pipeline();
        let job = test_job(&pipeline);
        let job_id = job.id().clone();

        repo.create(&job).await.expect("Failed to create");

        let found = repo.find_by_id(&job_id).await.expect("Failed to find");
        assert_eq!(found.id(), &job_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let fake_id = JobId::generate();
        assert!(repo.find_by_id(&fake_id).await.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline = test_pipeline();
        let mut job = test_job(&pipeline);

        repo.create(&job).await.expect("Failed to create");

        job.start().unwrap();
        let updated = repo.update(&job).await.expect("Failed to update");
        assert_eq!(
            updated.status(),
            domain::value_objects::job::JobStatus::Running
        );
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline = test_pipeline();
        let job = test_job(&pipeline);
        let job_id = job.id().clone();

        repo.create(&job).await.expect("Failed to create");
        repo.delete(&job_id).await.expect("Failed to delete");
        assert!(repo.find_by_id(&job_id).await.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline = test_pipeline();
        let j1 = test_job(&pipeline);
        let j2 = test_job(&pipeline);

        repo.create(&j1).await.unwrap();
        repo.create(&j2).await.unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert!(result.items().len() >= 2);
    }

    #[tokio::test]
    async fn test_list_by_pipeline() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline_a = test_pipeline();
        let pipeline_b = test_pipeline();

        let j1 = test_job(&pipeline_a);
        let j2 = test_job(&pipeline_b);

        repo.create(&j1).await.unwrap();
        repo.create(&j2).await.unwrap();

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_by_pipeline(pipeline_a.id(), Some(&pagination))
            .await
            .unwrap();

        assert_eq!(result.items().len(), 1);
        assert_eq!(result.items()[0].pipeline_id(), pipeline_a.id());
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_all_default_pagination() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let result = repo.list_all(None).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_by_pipeline_empty() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let pipeline_id = PipelineId::generate();
        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_by_pipeline(&pipeline_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_by_project_empty() {
        let db = setup().await;
        let repo = SurrealJobRepository::new(db);

        let project_id = ProjectId::generate();
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
        let repo = SurrealJobRepository::new(db);

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
