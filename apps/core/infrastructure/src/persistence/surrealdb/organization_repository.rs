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

impl OrganizationRepository for SurrealOrganizationRepository {
    fn create(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = DomainResult<Organization>> + Send {
        let db = self.db.clone();
        let organization = organization.clone();
        async move {
            let created: Option<Organization> = db
                .create(RecordId::new(
                    OrganizationId::table_name(),
                    organization.id().as_str(),
                ))
                .content(organization.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created
                .ok_or_else(|| DomainError::infrastructure("Create returned no record".to_string()))
        }
    }

    fn find_by_id(
        &self,
        id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<Organization>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            let result: Option<Organization> = db
                .select(RecordId::new(OrganizationId::table_name(), id.as_str()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("Organization", id.to_string()))
        }
    }

    fn find_by_name(
        &self,
        name: &OrganizationName,
    ) -> impl Future<Output = DomainResult<Organization>> + Send {
        let db = self.db.clone();
        let name_str = name.to_string();
        let table = OrganizationId::table_name().to_string();
        async move {
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
    }

    fn update(
        &self,
        organization: &Organization,
    ) -> impl Future<Output = DomainResult<Organization>> + Send {
        let db = self.db.clone();
        let organization = organization.clone();
        async move {
            let updated: Option<Organization> = db
                .update(RecordId::new(
                    OrganizationId::table_name(),
                    organization.id().as_str(),
                ))
                .content(organization.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| {
                DomainError::not_found("Organization", organization.id().to_string())
            })
        }
    }

    fn delete(&self, id: &OrganizationId) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            db.delete::<Option<Organization>>(RecordId::new(
                OrganizationId::table_name(),
                id.as_str(),
            ))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Organization>>> + Send {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = OrganizationId::table_name().to_string();
        async move {
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
    }

    fn list_active(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<Organization>>> + Send {
        let db = self.db.clone();
        let params = pagination.cloned().unwrap_or_default();
        let table = OrganizationId::table_name().to_string();
        async move {
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
    }

    fn name_exists(
        &self,
        name: &OrganizationName,
    ) -> impl Future<Output = DomainResult<bool>> + Send {
        let db = self.db.clone();
        let name_str = name.to_string();
        let table = OrganizationId::table_name().to_string();
        async move {
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
}
