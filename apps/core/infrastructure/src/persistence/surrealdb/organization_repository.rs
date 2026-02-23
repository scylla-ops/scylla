use async_trait::async_trait;
use domain::entities::{Organization, OrganizationId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::OrganizationRepository;
use domain::value_objects::organization::OrganizationName;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;

pub struct SurrealOrganizationRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealOrganizationRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
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
