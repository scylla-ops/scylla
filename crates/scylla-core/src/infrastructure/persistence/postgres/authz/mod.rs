use crate::application::permission::entity_provider::{
    AuthzEntityProvider, PrincipalAuthz, ResourceAncestors,
};
use crate::domain::entities::{AppId, OrganizationId, PipelineId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::ResourceRef;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

/// Loads the authz facts Cedar needs from the existing membership tables and
/// tenancy foreign keys. Read-only; one query per dimension, no caching (fine
/// at current scale — revisit if check latency matters).
#[derive(Clone)]
pub struct PgAuthzEntityProvider {
    pool: PgPool,
}

impl PgAuthzEntityProvider {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthzEntityProvider for PgAuthzEntityProvider {
    #[instrument(skip(self), fields(user_id = %user))]
    async fn principal_authz(&self, user: &UserId) -> DomainResult<PrincipalAuthz> {
        let role_rows = sqlx::query!(
            "SELECT role_name FROM user_roles WHERE user_id = $1",
            user.as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        let roles = role_rows
            .into_iter()
            .map(|r| RoleName::new(r.role_name))
            .collect::<DomainResult<Vec<_>>>()?;

        let org_rows = sqlx::query!(
            "SELECT organization_id FROM user_organization WHERE user_id = $1",
            user.as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        let member_orgs = org_rows
            .into_iter()
            .map(|r| OrganizationId::new(r.organization_id))
            .collect();

        let proj_rows = sqlx::query!(
            "SELECT project_id FROM user_project WHERE user_id = $1",
            user.as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        let member_projects = proj_rows
            .into_iter()
            .map(|r| ProjectId::new(r.project_id))
            .collect();

        Ok(PrincipalAuthz {
            roles,
            member_orgs,
            member_projects,
        })
    }

    #[instrument(skip(self))]
    async fn resource_ancestors(&self, resource: &ResourceRef) -> DomainResult<ResourceAncestors> {
        match resource {
            ResourceRef::Project(id) => {
                let row = sqlx::query!(
                    "SELECT organization_id FROM projects WHERE id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                Ok(ResourceAncestors {
                    organization: row.map(|r| OrganizationId::new(r.organization_id)),
                    ..Default::default()
                })
            }
            ResourceRef::Pipeline(id) => {
                // Joined columns: sqlx can't prove the inner join yields a row,
                // so force NOT NULL with `!` — both FKs are NOT NULL in schema.
                let row = sqlx::query!(
                    "SELECT pl.project_id AS \"project_id!\", pr.organization_id AS \"organization_id!\" \
                     FROM pipelines pl JOIN projects pr ON pr.id = pl.project_id \
                     WHERE pl.id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(r.organization_id)),
                        project: Some(ProjectId::new(r.project_id)),
                        pipeline: None,
                    }),
                    None => Ok(ResourceAncestors::default()),
                }
            }
            ResourceRef::Job(id) => {
                let row = sqlx::query!(
                    "SELECT j.pipeline_id AS \"pipeline_id!\", pl.project_id AS \"project_id!\", \
                            pr.organization_id AS \"organization_id!\" \
                     FROM jobs j \
                     JOIN pipelines pl ON pl.id = j.pipeline_id \
                     JOIN projects pr ON pr.id = pl.project_id \
                     WHERE j.id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(r.organization_id)),
                        project: Some(ProjectId::new(r.project_id)),
                        pipeline: Some(PipelineId::new(r.pipeline_id)),
                    }),
                    None => Ok(ResourceAncestors::default()),
                }
            }
            ResourceRef::App(id) => {
                let row = sqlx::query!(
                    "SELECT organization_id FROM apps WHERE id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                Ok(ResourceAncestors {
                    organization: row.map(|r| OrganizationId::new(r.organization_id)),
                    ..Default::default()
                })
            }
            // System / User / Organization have no tenancy parents.
            _ => Ok(ResourceAncestors::default()),
        }
    }

    #[instrument(skip(self), fields(app_id = %app))]
    async fn app_is_active(&self, app: &AppId) -> DomainResult<bool> {
        let row = sqlx::query!("SELECT is_active FROM apps WHERE id = $1", app.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(infra)?;
        // No row → the App was deleted; treat as inactive so an in-flight stream
        // from a now-removed App is denied rather than trusted.
        Ok(row.is_some_and(|r| r.is_active))
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
