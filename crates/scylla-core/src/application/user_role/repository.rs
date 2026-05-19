use crate::domain::entities::UserId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;

/// Read/write access to user ↔ role assignments.
///
/// Used at permission-check time by `CedarPermissionService` to materialise the
/// principal entity's parent set (e.g. `Scylla::User::"<id>" in Scylla::Role::"admin"`).
#[async_trait]
pub trait UserRoleRepository: Send + Sync {
    /// Roles directly assigned to this user. V1 has no role inheritance.
    async fn list_roles_for_user(&self, user_id: &UserId) -> DomainResult<Vec<RoleName>>;

    /// Grant a role to a user. Idempotent (no error if already assigned).
    async fn assign(&self, user_id: &UserId, role: &RoleName) -> DomainResult<()>;

    /// Revoke a role from a user. Idempotent (no error if absent).
    async fn revoke(&self, user_id: &UserId, role: &RoleName) -> DomainResult<()>;
}
