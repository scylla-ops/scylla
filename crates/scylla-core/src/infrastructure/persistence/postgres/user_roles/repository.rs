use crate::application::user_role::UserRoleRepository;
use crate::domain::entities::UserId;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::instrument;

#[derive(Clone)]
pub struct PgUserRoleRepository {
    pool: PgPool,
}

impl PgUserRoleRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRoleRepository for PgUserRoleRepository {
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn list_roles_for_user(&self, user_id: &UserId) -> DomainResult<Vec<RoleName>> {
        let rows = sqlx::query("SELECT role_name FROM user_roles WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                let name: String = r
                    .try_get("role_name")
                    .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
                RoleName::new(name)
            })
            .collect()
    }

    #[instrument(skip(self), fields(user_id = %user_id, role = %role))]
    async fn assign(&self, user_id: &UserId, role: &RoleName) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_name) VALUES ($1, $2) \
             ON CONFLICT (user_id, role_name) DO NOTHING",
        )
        .bind(user_id.as_str())
        .bind(role.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id, role = %role))]
    async fn revoke(&self, user_id: &UserId, role: &RoleName) -> DomainResult<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_name = $2")
            .bind(user_id.as_str())
            .bind(role.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        Ok(())
    }
}
