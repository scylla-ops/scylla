use async_trait::async_trait;
use domain::entities::{Organization, OrganizationId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::OrganizationRepository;
use domain::value_objects::organization::OrganizationName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;

pub struct SurrealOrganizationRepository {
    db: Surreal<Any>,
}

impl SurrealOrganizationRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl OrganizationRepository for SurrealOrganizationRepository {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization> {
        let db = self.db.clone();
        let organization = organization.clone();
        let created: Option<Organization> = db
            .create(RecordId::new(
                OrganizationId::table_name(),
                organization.id().as_str(),
            ))
            .content(organization.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
    }

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<Organization> = db
            .select(RecordId::new(OrganizationId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("Organization", id.to_string()))
    }

    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization> {
        let db = self.db.clone();
        let name_str = name.to_string();
        let table = OrganizationId::table_name().to_string();
        let mut results: Vec<Organization> = db
            .query("SELECT * FROM type::table($table) WHERE name = $name LIMIT 1")
            .bind(("table", table))
            .bind(("name", name_str.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results
            .pop()
            .ok_or_else(|| DomainError::not_found("Organization", name_str))
    }

    async fn update(&self, organization: &Organization) -> DomainResult<Organization> {
        let db = self.db.clone();
        let organization = organization.clone();
        let updated: Option<Organization> = db
            .update(RecordId::new(
                OrganizationId::table_name(),
                organization.id().as_str(),
            ))
            .content(organization.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| DomainError::not_found("Organization", organization.id().to_string()))
    }

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<Organization>>(RecordId::new(OrganizationId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = OrganizationId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let organizations: Vec<Organization> = db
                .query("SELECT * FROM type::table($table) ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(organizations, &params, total_count))
    }

    async fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = OrganizationId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE is_active = true GROUP ALL")
            .bind(("table", table.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let organizations: Vec<Organization> = db
                .query("SELECT * FROM type::table($table) WHERE is_active = true ORDER BY created_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(PaginatedResult::new(organizations, &params, total_count))
    }

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
        let db = self.db.clone();
        let name_str = name.to_string();
        let table = OrganizationId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE name = $name GROUP ALL")
            .bind(("table", table))
            .bind(("name", name_str))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let count = count_result.first().copied().unwrap_or(0);
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_db;
    use domain::entities::Organization;
    use domain::value_objects::PaginationParams;
    use domain::value_objects::organization::{OrganizationDescription, OrganizationName};

    async fn setup() -> Surreal<Any> {
        init_db(&[OrganizationId::table_name()]).await
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Test Organization").expect("Invalid name");
        let org = Organization::create(name, None).unwrap();
        let org_id = org.id().clone();

        let created = repo
            .create(&org)
            .await
            .expect("Failed to create organization");
        assert_eq!(created.id(), &org_id);
        assert_eq!(created.name(), org.name());
        assert_eq!(created.description(), org.description());
        assert_eq!(created.is_active(), org.is_active());
        assert_eq!(created.created_at(), org.created_at());
        assert_eq!(created.updated_at(), org.updated_at());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Find By Id Org").expect("Invalid name");
        let org = Organization::create(name, None).unwrap();
        let org_id = org.id().clone();

        repo.create(&org).await.expect("Failed to create");

        let found = repo
            .find_by_id(&org_id)
            .await
            .expect("Failed to find organization by id");
        assert_eq!(found.id(), &org_id);
        assert_eq!(found.name(), org.name());
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let fake_id = OrganizationId::generate();
        let result = repo.find_by_id(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Find By Name Org").expect("Invalid name");
        let org = Organization::create(name.clone(), None).unwrap();
        let org_id = org.id().clone();

        repo.create(&org).await.expect("Failed to create");

        let found = repo
            .find_by_name(&name)
            .await
            .expect("Failed to find organization by name");
        assert_eq!(found.id(), &org_id);
        assert_eq!(found.name(), &name);
    }

    #[tokio::test]
    async fn test_find_by_name_not_found() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Non Existent Org").expect("Invalid name");
        let result = repo.find_by_name(&name).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Update Org").expect("Invalid name");
        let mut org = Organization::create(name, None).unwrap();
        let org_id = org.id().clone();

        repo.create(&org).await.expect("Failed to create");

        let new_name = OrganizationName::new("Updated Org Name").expect("Invalid name");
        org.update_name(new_name.clone()).unwrap();

        let desc = OrganizationDescription::new("A description").unwrap();
        org.update_description(Some(desc)).unwrap();

        let updated = repo.update(&org).await.expect("Failed to update");
        assert_eq!(updated.id(), &org_id);
        assert_eq!(updated.name(), &new_name);
        assert!(updated.description().is_some());
    }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Delete Org").expect("Invalid name");
        let org = Organization::create(name, None).unwrap();
        let org_id = org.id().clone();

        repo.create(&org).await.expect("Failed to create");

        repo.delete(&org_id).await.expect("Failed to delete");

        let result = repo.find_by_id(&org_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name1 = OrganizationName::new("List All Org 1").expect("Invalid name");
        let org1 = Organization::create(name1, None).unwrap();
        repo.create(&org1).await.expect("Failed to create org1");

        let name2 = OrganizationName::new("List All Org 2").expect("Invalid name");
        let org2 = Organization::create(name2, None).unwrap();
        repo.create(&org2).await.expect("Failed to create org2");

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
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("List All Default Org").expect("Invalid name");
        let org = Organization::create(name, None).unwrap();
        repo.create(&org).await.expect("Failed to create");

        let result = repo
            .list_all(None)
            .await
            .expect("Failed to list all with default pagination");
        assert!(!result.items().is_empty());
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_all(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_active_empty() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo.list_active(Some(&pagination)).await.unwrap();
        assert_eq!(result.items().len(), 0);
        assert_eq!(result.metadata().total_count(), 0);
    }

    #[tokio::test]
    async fn test_list_active() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name_active = OrganizationName::new("Active Org").expect("Invalid name");
        let org_active = Organization::create(name_active, None).unwrap();
        repo.create(&org_active)
            .await
            .expect("Failed to create active org");

        let name_inactive = OrganizationName::new("Inactive Org").expect("Invalid name");
        let mut org_inactive = Organization::create(name_inactive, None).unwrap();
        org_inactive.deactivate().unwrap();
        repo.create(&org_inactive)
            .await
            .expect("Failed to create inactive org");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_active(Some(&pagination))
            .await
            .expect("Failed to list active");

        // All returned items must be active
        for item in result.items() {
            assert!(item.is_active());
        }
        // The active org should be in the results
        assert!(result.items().iter().any(|o| o.id() == org_active.id()));
    }

    #[tokio::test]
    async fn test_name_exists() {
        let db = setup().await;
        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Exists Org").expect("Invalid name");
        let org = Organization::create(name.clone(), None).unwrap();
        repo.create(&org).await.expect("Failed to create");

        let exists = repo
            .name_exists(&name)
            .await
            .expect("Failed to check name_exists");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_name_exists_false() {
        let db = setup().await;

        let repo = SurrealOrganizationRepository::new(db);

        let name = OrganizationName::new("Does Not Exist Org").expect("Invalid name");
        let exists = repo
            .name_exists(&name)
            .await
            .expect("Failed to check name_exists");
        assert!(!exists);
    }
}
