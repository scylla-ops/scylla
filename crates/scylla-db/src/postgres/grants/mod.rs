use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, OrganizationId, ProjectId, UserId};
use crate::domain::role::RoleName;
use async_trait::async_trait;
use scylla_auth::authz::{Grant, GrantRepository, Principal, Scope};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::error::SqlxResultExt;

const SCOPE_SYSTEM: &str = "system";
const SCOPE_ORGANIZATION: &str = "organization";
const SCOPE_PROJECT: &str = "project";
/// The column is NOT NULL and there is one System root, so the id is constant and ignored on read.
const SYSTEM_SCOPE_ID: &str = "system";
const PRINCIPAL_USER: &str = "user";
const PRINCIPAL_APP: &str = "app";

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

/// System-scoped grants stay out of reach: an org admin must not strip a platform operator.
pub async fn delete_under_scope<'e, E>(
    executor: E,
    principal: &Principal,
    scope: &Scope,
) -> DomainResult<u64>
where
    E: PgExecutor<'e>,
{
    let result = match scope {
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

    #[instrument(skip_all, fields(grant_id = %grant.id))]
    async fn create(&self, grant: &Grant) -> DomainResult<()> {
        insert(&self.pool, grant).await
    }

    #[instrument(skip_all, fields(principal = %principal, scope = %scope))]
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
    use crate::domain::ids::AppId;
    use crate::domain::role::RoleName;
    use crate::test_support::prelude::*;
    use scylla_auth::authz::{
        Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, ORGANIZATION_AGENT_ROLE,
        PROJECT_ADMIN_ROLE, Principal, SYSTEM_ADMIN_ROLE, Scope,
    };
    use sqlx::PgPool;

    fn role(name: &str) -> RoleName {
        RoleName::new(name).unwrap()
    }

    async fn seed_app(pool: &PgPool, org: &crate::domain::organization::Organization) -> AppId {
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

    /// `principal_id` and `scope_id` are polymorphic, so triggers, not FK cascades, clear grants.
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
