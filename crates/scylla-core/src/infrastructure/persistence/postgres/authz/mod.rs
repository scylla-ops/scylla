use crate::application::permission::entity_provider::{
    AuthzEntityProvider, PrincipalAuthz, ResourceAncestors,
};
use crate::domain::entities::{OrganizationId, PipelineId, ProjectId, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::ResourceRef;
use crate::domain::value_objects::role::name::RoleName;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
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
        let role_rows = sqlx::query("SELECT role_name FROM user_roles WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(infra)?;
        let roles = role_rows
            .iter()
            .map(|r| {
                let name: String = r.try_get("role_name").map_err(infra)?;
                RoleName::new(name)
            })
            .collect::<DomainResult<Vec<_>>>()?;

        let org_rows =
            sqlx::query("SELECT organization_id FROM user_organization WHERE user_id = $1")
                .bind(user.as_str())
                .fetch_all(&self.pool)
                .await
                .map_err(infra)?;
        let member_orgs = org_rows
            .iter()
            .map(|r| -> DomainResult<OrganizationId> {
                Ok(OrganizationId::new(
                    r.try_get::<String, _>("organization_id").map_err(infra)?,
                ))
            })
            .collect::<DomainResult<Vec<_>>>()?;

        let proj_rows = sqlx::query("SELECT project_id FROM user_project WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(infra)?;
        let member_projects = proj_rows
            .iter()
            .map(|r| -> DomainResult<ProjectId> {
                Ok(ProjectId::new(
                    r.try_get::<String, _>("project_id").map_err(infra)?,
                ))
            })
            .collect::<DomainResult<Vec<_>>>()?;

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
                let row = sqlx::query("SELECT organization_id FROM projects WHERE id = $1")
                    .bind(id.as_str())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(infra)?;
                let organization = match row {
                    Some(r) => Some(OrganizationId::new(
                        r.try_get::<String, _>("organization_id").map_err(infra)?,
                    )),
                    None => None,
                };
                Ok(ResourceAncestors {
                    organization,
                    ..Default::default()
                })
            }
            ResourceRef::Pipeline(id) => {
                let row = sqlx::query(
                    "SELECT pl.project_id, pr.organization_id \
                     FROM pipelines pl JOIN projects pr ON pr.id = pl.project_id \
                     WHERE pl.id = $1",
                )
                .bind(id.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(
                            r.try_get::<String, _>("organization_id").map_err(infra)?,
                        )),
                        project: Some(ProjectId::new(
                            r.try_get::<String, _>("project_id").map_err(infra)?,
                        )),
                        pipeline: None,
                    }),
                    None => Ok(ResourceAncestors::default()),
                }
            }
            ResourceRef::Job(id) => {
                let row = sqlx::query(
                    "SELECT j.pipeline_id, pl.project_id, pr.organization_id \
                     FROM jobs j \
                     JOIN pipelines pl ON pl.id = j.pipeline_id \
                     JOIN projects pr ON pr.id = pl.project_id \
                     WHERE j.id = $1",
                )
                .bind(id.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(infra)?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(
                            r.try_get::<String, _>("organization_id").map_err(infra)?,
                        )),
                        project: Some(ProjectId::new(
                            r.try_get::<String, _>("project_id").map_err(infra)?,
                        )),
                        pipeline: Some(PipelineId::new(
                            r.try_get::<String, _>("pipeline_id").map_err(infra)?,
                        )),
                    }),
                    None => Ok(ResourceAncestors::default()),
                }
            }
            ResourceRef::App(id) => {
                let row = sqlx::query("SELECT organization_id FROM apps WHERE id = $1")
                    .bind(id.as_str())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(infra)?;
                let organization = match row {
                    Some(r) => Some(OrganizationId::new(
                        r.try_get::<String, _>("organization_id").map_err(infra)?,
                    )),
                    None => None,
                };
                Ok(ResourceAncestors {
                    organization,
                    ..Default::default()
                })
            }
            // System / User / Organization have no tenancy parents.
            _ => Ok(ResourceAncestors::default()),
        }
    }
}

fn infra<E: std::fmt::Display>(e: E) -> DomainError {
    DomainError::Infrastructure(e.to_string())
}
