use crate::domain::entities::UserOrganization;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserOrganizationRepository;
use crate::domain::value_objects::{OrganizationId, UserId, UserOrganizationId};
use crate::infrastructure::persistence::mappers::{FromRecordId, ToRecordId};
use crate::infrastructure::persistence::surrealdb::models::UserOrganizationRecord;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB implementation of OrganizationRepository
#[derive(Constructor)]
pub struct SurrealUserOrganizationRepository {
    db: Arc<Surreal<Any>>,
}

#[async_trait]
impl UserOrganizationRepository for SurrealUserOrganizationRepository {
    async fn create(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization> {
        let created: Option<UserOrganizationRecord> = self
            .db
            .query("RELATE $user_id->user_organization->$organization_id SET role = $role")
            .bind(("user_id", user_organization.user_id().to_record_id()))
            .bind((
                "organization_id",
                user_organization.organization_id().to_record_id(),
            ))
            .bind(("role", user_organization.role().to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match created {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure(
                "Failed to create user organization",
            )),
        }
    }

    async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization> {
        let result: Option<UserOrganizationRecord> = self
            .db
            .select(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found("User organization", id.to_string())),
        }
    }

    async fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<UserOrganization> {
        let results: Vec<UserOrganizationRecord> = self
            .db
            .query("SELECT * FROM user_organization WHERE in = $user_id AND out = $organization_id")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("organization_id", organization_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let result = results.into_iter().next();

        match result {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::not_found(
                "User organization",
                format!(
                    "user_id: {}, organization_id: {}",
                    user_id.to_string(),
                    organization_id.to_string()
                ),
            )),
        }
    }

    async fn update(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization> {
        let results: Vec<UserOrganizationRecord> = self
			.db
			.query("UPDATE user_organization SET role = $role WHERE in = $user_id AND out = $organization_id")
			.bind(("user_id", user_organization.user_id().to_record_id()))
			.bind((
				"organization_id",
				user_organization.organization_id().to_record_id(),
			))
			.bind(("role", user_organization.role().to_string()))
			.await
			.map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
			.take(0)
			.map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let updated = results.into_iter().next();

        match updated {
            Some(record) => Ok(record.try_into()?),
            None => Err(DomainError::infrastructure(
                "Failed to update user organization",
            )),
        }
    }

    async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()> {
        self.db
            .delete::<Option<UserOrganizationRecord>>(id.to_record_id())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(&self) -> DomainResult<Vec<UserOrganization>> {
        let records: Vec<UserOrganizationRecord> = self
            .db
            .select("user_organizations")
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        records
            .into_iter()
            .map(|record| record.try_into())
            .collect()
    }

    async fn list_organizations_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<OrganizationId>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM $user_id->user_organization GROUP ALL")
            .bind(("user_id", user_id.to_record_id()))
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
        let records: Vec<UserOrganizationRecord> = self
            .db
            .query("SELECT * FROM $user_id->user_organization LIMIT $limit START $start")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let organization_ids: DomainResult<Vec<OrganizationId>> = records
            .into_iter()
            .map(|record| Ok(OrganizationId::from_record_id(record.organization_id)))
            .collect();

        Ok(PaginatedResult::new(
            organization_ids?,
            &params,
            total_count,
        ))
    }

    async fn list_users_in_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&crate::domain::value_objects::PaginationParams>,
    ) -> DomainResult<crate::domain::value_objects::PaginatedResult<UserId>> {
        use crate::domain::value_objects::{PaginatedResult, PaginationParams};

        let params = pagination
            .cloned()
            .unwrap_or_else(PaginationParams::default);

        // Get total count
        let count_result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM $organization_id<-user_organization GROUP ALL")
            .bind(("organization_id", organization_id.to_record_id()))
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
        let records: Vec<UserOrganizationRecord> = self
            .db
            .query("SELECT * FROM $organization_id<-user_organization LIMIT $limit START $start")
            .bind(("organization_id", organization_id.to_record_id()))
            .bind(("limit", params.limit()))
            .bind(("start", params.offset()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let user_ids: DomainResult<Vec<UserId>> = records
            .into_iter()
            .map(|record| Ok(UserId::from_record_id(record.user_id)))
            .collect();

        Ok(PaginatedResult::new(user_ids?, &params, total_count))
    }

    async fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId> {
        let created: Option<UserOrganizationRecord> = self
            .db
            .query("RELATE $user_id->user_organization->$organization_id SET role = $role")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("organization_id", organization_id.to_record_id()))
            .bind(("role", role.to_string()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        match created {
            Some(record) => Ok(UserOrganizationId::from_record_id(record.id)),
            None => Err(DomainError::infrastructure(
                "Failed to add user to organization",
            )),
        }
    }

    async fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<()> {
        let _: Vec<UserOrganizationRecord> = self
            .db
            .query("DELETE user_organization WHERE in = $user_id AND out = $organization_id")
            .bind(("user_id", user_id.to_record_id()))
            .bind(("organization_id", organization_id.to_record_id()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(())
    }
}
