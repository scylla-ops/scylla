use domain::entities::{OrganizationId, UserId, UserOrganization, UserOrganizationId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserOrganizationRepository;
use domain::value_objects::user_organization::user_organization_role::UserOrganizationRole;
use domain::value_objects::{PaginatedResult, PaginationParams};
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;

pub struct SurrealUserOrganizationRepository {
    db: Arc<Surreal<Any>>,
}

impl SurrealUserOrganizationRepository {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }
}

impl UserOrganizationRepository for SurrealUserOrganizationRepository {
    fn create(
        &self,
        user_organization: &UserOrganization,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send {
        let db = self.db.clone();
        let user_organization = user_organization.clone();
        async move {
            let created: Option<UserOrganization> = db
                .create(RecordId::new(
                    UserOrganizationId::table_name(),
                    user_organization.id().as_str(),
                ))
                .content(user_organization.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            created.ok_or_else(|| DomainError::infrastructure("Failed to create user organization"))
        }
    }

    fn find_by_id(
        &self,
        id: &UserOrganizationId,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            let result: Option<UserOrganization> = db
                .select(RecordId::new(
                    UserOrganizationId::table_name(),
                    id.as_str(),
                ))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            result.ok_or_else(|| DomainError::not_found("UserOrganization", id.to_string()))
        }
    }

    fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let organization_id_str = organization_id.to_string();
        let table = UserOrganizationId::table_name().to_string();
        async move {
            let mut results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id AND organization_id = $organization_id LIMIT 1")
                .bind(("table", table))
                .bind(("user_id", user_id_str.clone()))
                .bind(("organization_id", organization_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            results.pop().ok_or_else(|| {
                DomainError::not_found(
                    "UserOrganization",
                    format!(
                        "user_id={}, organization_id={}",
                        user_id_str, organization_id_str
                    ),
                )
            })
        }
    }

    fn update(
        &self,
        user_organization: &UserOrganization,
    ) -> impl Future<Output = DomainResult<UserOrganization>> + Send {
        let db = self.db.clone();
        let user_organization = user_organization.clone();
        async move {
            let updated: Option<UserOrganization> = db
                .update(RecordId::new(
                    UserOrganizationId::table_name(),
                    user_organization.id().as_str(),
                ))
                .content(user_organization.clone())
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            updated.ok_or_else(|| {
                DomainError::not_found("UserOrganization", user_organization.id().to_string())
            })
        }
    }

    fn delete(&self, id: &UserOrganizationId) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let id = id.clone();
        async move {
            db.delete::<Option<UserOrganization>>(RecordId::new(
                UserOrganizationId::table_name(),
                id.as_str(),
            ))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }

    fn list_all(&self) -> impl Future<Output = DomainResult<Vec<UserOrganization>>> + Send {
        let db = self.db.clone();
        let table = UserOrganizationId::table_name().to_string();
        async move {
            let results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table)")
                .bind(("table", table))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            Ok(results)
        }
    }

    fn list_organizations_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<OrganizationId>>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserOrganizationId::table_name().to_string();
        async move {
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE user_id = $user_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("user_id", user_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let org_ids: Vec<OrganizationId> = results
                .into_iter()
                .map(|uo| uo.organization_id().clone())
                .collect();

            Ok(PaginatedResult::new(org_ids, &params, total_count))
        }
    }

    fn list_users_in_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> impl Future<Output = DomainResult<PaginatedResult<UserId>>> + Send {
        let db = self.db.clone();
        let organization_id_str = organization_id.to_string();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserOrganizationId::table_name().to_string();
        async move {
            let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE organization_id = $organization_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("organization_id", organization_id_str.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let total_count = count_result.first().copied().unwrap_or(0) as u64;

            let results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE organization_id = $organization_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("organization_id", organization_id_str))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

            let user_ids: Vec<UserId> =
                results.into_iter().map(|uo| uo.user_id().clone()).collect();

            Ok(PaginatedResult::new(user_ids, &params, total_count))
        }
    }

    fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> impl Future<Output = DomainResult<UserOrganizationId>> + Send {
        let db = self.db.clone();
        let user_id = user_id.clone();
        let organization_id = organization_id.clone();
        let role = role.to_string();
        async move {
            let parsed_role = UserOrganizationRole::new(&role)?;

            let user_org = UserOrganization::create(user_id, organization_id, parsed_role)?;
            let id = user_org.id().clone();

            let _created: Option<UserOrganization> = db
                .create(RecordId::new(
                    UserOrganizationId::table_name(),
                    user_org.id().as_str(),
                ))
                .content(user_org)
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(id)
        }
    }

    fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> impl Future<Output = DomainResult<()>> + Send {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();
        let organization_id_str = organization_id.to_string();
        let table = UserOrganizationId::table_name().to_string();
        async move {
            db.query("DELETE FROM type::table($table) WHERE user_id = $user_id AND organization_id = $organization_id")
                .bind(("table", table))
                .bind(("user_id", user_id_str))
                .bind(("organization_id", organization_id_str))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

            Ok(())
        }
    }
}
