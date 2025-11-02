use crate::domain::entities::Organization;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::OrganizationRepository;
use crate::domain::value_objects::{OrganizationId, OrganizationName};
use crate::infrastructure::persistence::mappers::ToRecordId;
use crate::infrastructure::persistence::surrealdb::mappers::OrganizationMapper;
use crate::infrastructure::persistence::surrealdb::models::OrganizationRecord;
use async_trait::async_trait;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of OrganizationRepository
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
        let insert = OrganizationMapper::to_insert(organization);
        let created: Option<OrganizationRecord> = self
            .db
            .create("organizations")
            .content(insert)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match created {
            Some(record) => Ok(OrganizationMapper::to_domain(record)?),
            None => Err(DomainError::infrastructure("Failed to create organization")),
        }
    }

    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
        let result: Option<OrganizationRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(OrganizationMapper::to_domain(record)?),
            None => Err(DomainError::not_found("Organization", id.to_string())),
        }
    }

    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization> {
        let mut results: Vec<OrganizationRecord> = self
            .db
            .query("SELECT * FROM organizations WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match results.pop() {
            Some(record) => Ok(OrganizationMapper::to_domain(record)?),
            None => Err(DomainError::not_found("Organization", name.to_string())),
        }
    }

    async fn update(&self, organization: &Organization) -> DomainResult<Organization> {
        let record = OrganizationMapper::to_update(organization);
        let updated: Option<OrganizationRecord> = self
            .db
            .update(organization.id().to_record_id())
            .merge(record)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match updated {
            Some(record) => Ok(OrganizationMapper::to_domain(record)?),
            None => Err(DomainError::infrastructure("Failed to update organization")),
        }
    }

    async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.db
            .delete::<Option<OrganizationRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Organization>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM organizations GROUP ALL")
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<OrganizationRecord> = self
            .db
            .query("SELECT * FROM organizations ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let organizations: DomainResult<Vec<Organization>> = records
            .into_iter()
            .map(|record| OrganizationMapper::to_domain(record))
            .collect();

        Ok(PaginatedResult::new(organizations?, &params, total_count))
    }

    async fn list_active(
        &self,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<Organization>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM organizations WHERE is_active = true GROUP ALL")
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Get paginated records
        let records: Vec<OrganizationRecord> = self
            .db
            .query("SELECT * FROM organizations WHERE is_active = true ORDER BY created_at DESC LIMIT $limit START $start")
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let organizations: DomainResult<Vec<Organization>> = records
            .into_iter()
            .map(|record| OrganizationMapper::to_domain(record))
            .collect();

        Ok(PaginatedResult::new(organizations?, &params, total_count))
    }

    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
        match self.find_by_name(name).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
