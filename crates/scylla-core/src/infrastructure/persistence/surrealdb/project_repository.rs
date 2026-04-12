use crate::application::ports::ProjectRepository;
use crate::domain::entities::{OrganizationId, Project, ProjectId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use async_trait::async_trait;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;
use tracing::instrument;

pub struct SurrealProjectRepository {
    db: Surreal<Any>,
}

impl SurrealProjectRepository {
    #[must_use]
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepository for SurrealProjectRepository {
    #[instrument(skip(self, project), fields(project_id = %project.id()))]
    async fn create(&self, project: &Project) -> DomainResult<Project> {
        let db = self.db.clone();
        let project = project.clone();
        let created: Option<Project> = db
            .create(RecordId::new(
                ProjectId::table_name(),
                project.id().as_str(),
            ))
            .content(project.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    #[instrument(skip(self), fields(project_id = %id))]
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<Project> = db
            .select(RecordId::new(ProjectId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        result.ok_or_else(|| DomainError::not_found("Project", id.to_string()))
    }

    #[instrument(skip(self, project), fields(project_id = %project.id()))]
    async fn update(&self, project: &Project) -> DomainResult<Project> {
        let db = self.db.clone();
        let project = project.clone();
        let updated: Option<Project> = db
            .update(RecordId::new(
                ProjectId::table_name(),
                project.id().as_str(),
            ))
            .content(project.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        updated.ok_or_else(|| DomainError::not_found("Project", project.id().to_string()))
    }

    #[instrument(skip(self), fields(project_id = %id))]
    async fn delete(&self, id: &ProjectId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<Project>>(RecordId::new(ProjectId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let projects: Vec<Project> = db
                .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(projects, &params, total_count))
    }

    #[instrument(skip(self))]
    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE is_active = true GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let projects: Vec<Project> = db
                .query("SELECT * FROM type::table($table) WHERE is_active = true ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(projects, &params, total_count))
    }

    #[instrument(skip(self), fields(organization_id = %organization_id))]
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        let db = self.db.clone();
        let params = pagination.copied().unwrap_or_default();
        let table = ProjectId::table_name().to_string();
        let organization_id = organization_id.clone();

        let count_result: Vec<i64> = db
            .query(
                "SELECT count() FROM type::table($table) WHERE organization_id = $org_id GROUP ALL",
            )
            .bind(("table", table.clone()))
            .bind(("org_id", organization_id.clone().into_value()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        let total_count = count_result.first().copied().unwrap_or(0).cast_unsigned();

        let projects: Vec<Project> = db
            .query("SELECT * FROM type::table($table) WHERE organization_id = $org_id ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("table", table))
            .bind(("org_id", organization_id.into_value()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {e}")))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {e}")))?;

        Ok(PaginatedResult::new(projects, &params, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{OrganizationId, Project};
    use crate::domain::value_objects::PaginationParams;
    use crate::domain::value_objects::project::{ProjectDescription, ProjectName};
    use crate::infrastructure::test_utils::init_db;

    async fn setup() -> Surreal<Any> {
        init_db(&[ProjectId::table_name()]).await
    }

    fn test_org_id() -> OrganizationId {
        OrganizationId::generate()
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let name = ProjectName::new("Test Project").expect("Invalid name");
        let org_id = test_org_id();
        let project = Project::create(name, None, org_id).unwrap();
        let project_id = project.id().clone();

        let created = repo
            .create(&project)
            .await
            .expect("Failed to create project");
        assert_eq!(created.id(), &project_id);
        assert_eq!(created.name(), project.name());
        assert_eq!(created.description(), project.description());
        assert_eq!(created.organization_id(), project.organization_id());
        assert_eq!(created.is_active(), project.is_active());
        assert_eq!(created.created_at(), project.created_at());
        assert_eq!(created.updated_at(), project.updated_at());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let name = ProjectName::new("Find By Id Project").expect("Invalid name");
        let org_id = test_org_id();
        let project = Project::create(name, None, org_id).unwrap();
        let project_id = project.id().clone();

        repo.create(&project).await.expect("Failed to create");

        let found = repo
            .find_by_id(&project_id)
            .await
            .expect("Failed to find project by id");
        assert_eq!(found.id(), &project_id);
        assert_eq!(found.name(), project.name());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let fake_id = ProjectId::generate();
        let result = repo.find_by_id(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let name = ProjectName::new("Update Project").expect("Invalid name");
        let org_id = test_org_id();
        let mut project = Project::create(name, None, org_id).unwrap();
        let project_id = project.id().clone();

        repo.create(&project).await.expect("Failed to create");

        let new_name = ProjectName::new("Updated Project Name").expect("Invalid name");
        project.update_name(new_name.clone()).unwrap();

        let desc = ProjectDescription::new("A description").unwrap();
        project.update_description(Some(desc)).unwrap();

        let updated = repo.update(&project).await.expect("Failed to update");
        assert_eq!(updated.id(), &project_id);
        assert_eq!(updated.name(), &new_name);
        assert!(updated.description().is_some());
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let name = ProjectName::new("Delete Project").expect("Invalid name");
        let org_id = test_org_id();
        let project = Project::create(name, None, org_id).unwrap();
        let project_id = project.id().clone();

        repo.create(&project).await.expect("Failed to create");

        repo.delete(&project_id).await.expect("Failed to delete");

        let result = repo.find_by_id(&project_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);
        let org_id = test_org_id();

        let name1 = ProjectName::new("List All Project 1").expect("Invalid name");
        let project1 = Project::create(name1, None, org_id.clone()).unwrap();
        repo.create(&project1)
            .await
            .expect("Failed to create project1");

        let name2 = ProjectName::new("List All Project 2").expect("Invalid name");
        let project2 = Project::create(name2, None, org_id).unwrap();
        repo.create(&project2)
            .await
            .expect("Failed to create project2");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_all(Some(&pagination))
            .await
            .expect("Failed to list all");
        assert!(result.items().len() >= 2);
        assert!(result.metadata().total_count() >= 2);
    }

    #[tokio::test]
    async fn test_list_all_default_pagination() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let name = ProjectName::new("List All Default Project").expect("Invalid name");
        let org_id = test_org_id();
        let project = Project::create(name, None, org_id).unwrap();
        repo.create(&project).await.expect("Failed to create");

        let result = repo
            .list_all(None)
            .await
            .expect("Failed to list all with default pagination");
        assert!(!result.items().is_empty());
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_active_empty() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_active(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_active() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);
        let org_id = test_org_id();

        let name_active = ProjectName::new("Active Project").expect("Invalid name");
        let project_active = Project::create(name_active, None, org_id.clone()).unwrap();
        repo.create(&project_active)
            .await
            .expect("Failed to create active project");

        let name_inactive = ProjectName::new("Inactive Project").expect("Invalid name");
        let mut project_inactive = Project::create(name_inactive, None, org_id).unwrap();
        project_inactive.deactivate().unwrap();
        repo.create(&project_inactive)
            .await
            .expect("Failed to create inactive project");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_active(Some(&pagination))
            .await
            .expect("Failed to list active");

        // All returned items must be active
        for item in result.items() {
            assert!(item.is_active());
        }
        // The active project should be in the results
        assert!(result.items().iter().any(|p| p.id() == project_active.id()));
    }

    #[tokio::test]
    async fn test_list_by_organization() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let org_a = test_org_id();
        let org_b = test_org_id();

        let project_a1 = Project::create(
            ProjectName::new("Org A Project 1").unwrap(),
            None,
            org_a.clone(),
        )
        .unwrap();
        repo.create(&project_a1).await.expect("Failed to create");

        let project_a2 = Project::create(
            ProjectName::new("Org A Project 2").unwrap(),
            None,
            org_a.clone(),
        )
        .unwrap();
        repo.create(&project_a2).await.expect("Failed to create");

        let project_b = Project::create(
            ProjectName::new("Org B Project").unwrap(),
            None,
            org_b.clone(),
        )
        .unwrap();
        repo.create(&project_b).await.expect("Failed to create");

        let pagination = PaginationParams::new(1, 20).unwrap();

        // Org A should have 2 projects
        let result_a = repo
            .list_by_organization(&org_a, Some(&pagination))
            .await
            .expect("Failed to list by org A");
        assert_eq!(result_a.items().len(), 2);
        assert_eq!(result_a.metadata().total_count(), 2);
        for p in result_a.items() {
            assert_eq!(p.organization_id(), &org_a);
        }

        // Org B should have 1 project
        let result_b = repo
            .list_by_organization(&org_b, Some(&pagination))
            .await
            .expect("Failed to list by org B");
        assert_eq!(result_b.items().len(), 1);
        assert_eq!(result_b.metadata().total_count(), 1);
        assert_eq!(result_b.items()[0].organization_id(), &org_b);
    }

    #[tokio::test]
    async fn test_list_by_organization_empty() {
        let db = setup().await;
        let repo = SurrealProjectRepository::new(db);

        let org_id = test_org_id();
        let pagination = PaginationParams::new(1, 20).unwrap();

        let result = repo
            .list_by_organization(&org_id, Some(&pagination))
            .await
            .unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }
}
