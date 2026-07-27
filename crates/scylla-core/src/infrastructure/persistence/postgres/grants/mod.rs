use crate::application::authz::grant::{Grant, GrantRepository, Principal, Scope};
use crate::domain::entities::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::role::RoleName;
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::error::SqlxResultExt;

const SCOPE_SYSTEM: &str = "system";
const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";
/// Sentinel `scope_id` for the singleton System scope (column is NOT NULL; there
/// is only one System root, so the id is constant and ignored on read).
const SYSTEM_SCOPE_ID: &str = "system";
const PRINCIPAL_USER: &str = "user";
const PRINCIPAL_APP: &str = "app";

/// Insert a grant on any executor (pool or transaction). Idempotent via the
/// `(principal_kind, principal_id, role_id, scope_kind, scope_id)` unique
/// constraint, so re-running a signup or grant call is a no-op rather than a
/// conflict. Shared by the pool-backed repo and the atomic signup transaction.
pub async fn insert<'e, E>(executor: E, grant: &Grant) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match &grant.scope {
        Scope::System => (SCOPE_SYSTEM, SYSTEM_SCOPE_ID),
        Scope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        Scope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query!(
        "INSERT INTO grants (id, principal_kind, principal_id, role_id, scope_kind, scope_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (principal_kind, principal_id, role_id, scope_kind, scope_id) DO NOTHING",
        grant.id.as_str(),
        grant.principal.kind(),
        grant.principal.id(),
        grant.role.as_str(),
        scope_kind,
        scope_id,
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Delete every grant a principal holds at exactly `scope`, on any executor
/// (pool or transaction). Used when a member is removed from a project: the
/// Cedar member guard is what actually cuts an ex-member's access, but the
/// rows must go with the membership, or re-adding the user later would
/// silently restore their old authority.
pub async fn delete_by_principal_and_scope<'e, E>(
    executor: E,
    principal: &Principal,
    scope: &Scope,
) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    let (scope_kind, scope_id) = match scope {
        Scope::System => (SCOPE_SYSTEM, SYSTEM_SCOPE_ID),
        Scope::Organization(id) => (SCOPE_ORGANIZATION, id.as_str()),
        Scope::Project(id) => (SCOPE_PROJECT, id.as_str()),
    };
    sqlx::query!(
        "DELETE FROM grants \
         WHERE principal_kind = $1 AND principal_id = $2 \
           AND scope_kind = $3 AND scope_id = $4",
        principal.kind(),
        principal.id(),
        scope_kind,
        scope_id,
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Delete every grant a user holds anywhere under an organization — at the org
/// scope itself and at every project belonging to it — on any executor. Used
/// when a member is removed from an org: removing them from the org must strip
/// their authority over the whole subtree, not just the org level.
pub async fn delete_by_user_under_org<'e, E>(
    executor: E,
    user_id: &UserId,
    org_id: &OrganizationId,
) -> DomainResult<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "DELETE FROM grants \
         WHERE principal_kind = $1 AND principal_id = $2 \
           AND ((scope_kind = $3 AND scope_id = $4) \
             OR (scope_kind = $5 AND scope_id IN \
                   (SELECT id FROM projects WHERE organization_id = $4)))",
        PRINCIPAL_USER,
        user_id.as_str(),
        SCOPE_ORGANIZATION,
        org_id.as_str(),
        SCOPE_PROJECT,
    )
    .execute(executor)
    .await
    .to_domain()?;
    Ok(())
}

/// Delete every grant a principal holds at `scope` and everything beneath it.
/// System-scoped grants are deliberately out of reach: an organization
/// administrator revoking someone must not be able to strip a platform
/// operator's global access as a side effect.
pub async fn delete_under_scope<'e, E>(
    executor: E,
    principal: &Principal,
    scope: &Scope,
) -> DomainResult<u64>
where
    E: PgExecutor<'e>,
{
    let result = match scope {
        // Everything the principal holds except System itself.
        Scope::System => {
            sqlx::query!(
                "DELETE FROM grants \
                 WHERE principal_kind = $1 AND principal_id = $2 AND scope_kind <> $3",
                principal.kind(),
                principal.id(),
                SCOPE_SYSTEM,
            )
            .execute(executor)
            .await
        }
        // The org itself plus every project under it, in one statement.
        Scope::Organization(org_id) => {
            sqlx::query!(
                "DELETE FROM grants \
                 WHERE principal_kind = $1 AND principal_id = $2 \
                   AND ((scope_kind = $3 AND scope_id = $4) \
                     OR (scope_kind = $5 AND scope_id IN \
                           (SELECT id FROM projects WHERE organization_id = $4)))",
                principal.kind(),
                principal.id(),
                SCOPE_ORGANIZATION,
                org_id.as_str(),
                SCOPE_PROJECT,
            )
            .execute(executor)
            .await
        }
        Scope::Project(project_id) => {
            sqlx::query!(
                "DELETE FROM grants \
                 WHERE principal_kind = $1 AND principal_id = $2 \
                   AND scope_kind = $3 AND scope_id = $4",
                principal.kind(),
                principal.id(),
                SCOPE_PROJECT,
                project_id.as_str(),
            )
            .execute(executor)
            .await
        }
    };
    Ok(result.to_domain()?.rows_affected())
}

/// Persistence for explicit scoped grants (`grants` table). Read once
/// at `CedarPermissionService` construction to link template instances.
#[derive(Clone)]
pub struct PgGrantRepository {
    pool: PgPool,
}

impl PgGrantRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GrantRepository for PgGrantRepository {
    #[instrument(skip(self))]
    async fn list_all(&self) -> DomainResult<Vec<Grant>> {
        let rows = sqlx::query!(
            "SELECT id, principal_kind, principal_id, role_id, scope_kind, scope_id FROM grants",
        )
        .fetch_all(&self.pool)
        .await
        .to_domain()?;

        rows.into_iter()
            .map(|r| {
                let principal = match r.principal_kind.as_str() {
                    PRINCIPAL_USER => Principal::User(UserId::new(r.principal_id)),
                    PRINCIPAL_APP => Principal::App(AppId::new(r.principal_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant principal_kind '{other}'"
                        )));
                    }
                };
                let scope = match r.scope_kind.as_str() {
                    SCOPE_SYSTEM => Scope::System,
                    SCOPE_ORGANIZATION => Scope::Organization(OrganizationId::new(r.scope_id)),
                    SCOPE_PROJECT => Scope::Project(ProjectId::new(r.scope_id)),
                    other => {
                        return Err(DomainError::Infrastructure(format!(
                            "unknown grant scope_kind '{other}'"
                        )));
                    }
                };
                Ok(Grant {
                    id: r.id,
                    principal,
                    role: RoleName::new(r.role_id)?,
                    scope,
                })
            })
            .collect()
    }

    #[instrument(skip(self, grant), fields(grant_id = %grant.id))]
    async fn create(&self, grant: &Grant) -> DomainResult<()> {
        insert(&self.pool, grant).await
    }

    #[instrument(skip(self))]
    async fn revoke_all(&self, principal: &Principal, scope: &Scope) -> DomainResult<u64> {
        delete_under_scope(&self.pool, principal, scope).await
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> DomainResult<()> {
        sqlx::query!("DELETE FROM grants WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .to_domain()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PgGrantRepository;
    use crate::application::authz::grant::{
        Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, ORGANIZATION_AGENT_ROLE,
        PROJECT_ADMIN_ROLE, Principal, SYSTEM_ADMIN_ROLE, Scope,
    };
    use crate::domain::entities::AppId;
    use crate::domain::value_objects::role::RoleName;
    use crate::test_support::prelude::*;
    use sqlx::PgPool;

    fn role(name: &str) -> RoleName {
        RoleName::new(name).unwrap()
    }

    async fn seed_app(pool: &PgPool, org: &crate::domain::entities::Organization) -> AppId {
        let id = AppId::generate();
        sqlx::query!(
            "INSERT INTO apps (id, organization_id, name) VALUES ($1, $2, 'runner')",
            id.as_str(),
            org.id().as_str(),
        )
        .execute(pool)
        .await
        .unwrap();
        id
    }

    /// `grants.principal_id` / `scope_id` are polymorphic, so Postgres cannot
    /// cascade them; DB triggers do. Deleting an organization must clear every
    /// grant bound to it, to the projects it cascades away, and to the apps it
    /// cascades away — while another org's identical holdings survive.
    #[sqlx::test(migrations = "../../migrations")]
    async fn deleting_an_organization_clears_the_grants_of_its_whole_subtree(pool: PgPool) {
        let org = seed_org(&pool, "acme").await;
        let other_org = seed_org(&pool, "globex").await;
        let project = seed_project(&pool, &org, "apollo").await;
        let other_project = seed_project(&pool, &other_org, "zeus").await;
        let user = seed_user(&pool, "alice").await;
        let app = seed_app(&pool, &org).await;

        let repo = PgGrantRepository::new(pool.clone());
        let doomed = [
            Grant::new(
                Principal::User(user.id().clone()),
                role(ORGANIZATION_ADMIN_ROLE),
                Scope::Organization(org.id().clone()),
            ),
            Grant::new(
                Principal::User(user.id().clone()),
                role(PROJECT_ADMIN_ROLE),
                Scope::Project(project.id().clone()),
            ),
            // Held by an app of the doomed org: the app row cascades away, so the
            // principal-side trigger is what clears this one.
            Grant::new(
                Principal::App(app.clone()),
                role(ORGANIZATION_AGENT_ROLE),
                Scope::Organization(org.id().clone()),
            ),
        ];
        let survivors = [
            Grant::new(
                Principal::User(user.id().clone()),
                role(ORGANIZATION_ADMIN_ROLE),
                Scope::Organization(other_org.id().clone()),
            ),
            Grant::new(
                Principal::User(user.id().clone()),
                role(PROJECT_ADMIN_ROLE),
                Scope::Project(other_project.id().clone()),
            ),
        ];
        for g in doomed.iter().chain(survivors.iter()) {
            repo.create(g).await.unwrap();
        }

        sqlx::query!("DELETE FROM organizations WHERE id = $1", org.id().as_str())
            .execute(&pool)
            .await
            .unwrap();

        let remaining = repo.list_all().await.unwrap();
        for g in &doomed {
            assert!(
                !remaining.iter().any(|r| r.id == g.id),
                "grant {} should have gone with the organization",
                g.id,
            );
        }
        for g in &survivors {
            assert!(
                remaining.iter().any(|r| r.id == g.id),
                "grant {} belongs to another org and must survive",
                g.id,
            );
        }
    }

    /// Deleting a principal clears every grant it held, at any scope — including
    /// the System scope, which has no table to hang a foreign key on and no
    /// membership guard to make a leftover row inert.
    #[sqlx::test(migrations = "../../migrations")]
    async fn deleting_a_user_clears_their_grants_at_every_scope(pool: PgPool) {
        let org = seed_org(&pool, "acme").await;
        let user = seed_user(&pool, "alice").await;
        let keeper = seed_user(&pool, "bob").await;

        let repo = PgGrantRepository::new(pool.clone());
        let system = Grant::new(
            Principal::User(user.id().clone()),
            role(SYSTEM_ADMIN_ROLE),
            Scope::System,
        );
        let scoped = Grant::new(
            Principal::User(user.id().clone()),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(org.id().clone()),
        );
        let others = Grant::new(
            Principal::User(keeper.id().clone()),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(org.id().clone()),
        );
        for g in [&system, &scoped, &others] {
            repo.create(g).await.unwrap();
        }

        sqlx::query!("DELETE FROM users WHERE id = $1", user.id().as_str())
            .execute(&pool)
            .await
            .unwrap();

        let remaining = repo.list_all().await.unwrap();
        assert_eq!(
            remaining.len(),
            1,
            "only the other user's grant should be left",
        );
        assert_eq!(remaining[0].id, others.id);
    }
}
