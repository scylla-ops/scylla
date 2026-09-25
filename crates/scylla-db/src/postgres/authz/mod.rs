use crate::domain::errors::DomainResult;
use crate::domain::ids::{AppId, OrganizationId, PipelineId, ProjectId};
use crate::domain::permission::ResourceRef;
use async_trait::async_trait;
use scylla_auth::authz::{AuthzEntityProvider, ResourceAncestors};
use sqlx::PgPool;
use tracing::instrument;

use super::error::SqlxResultExt;

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
    #[instrument(skip_all, fields(resource = %resource))]
    async fn resource_ancestors(&self, resource: &ResourceRef) -> DomainResult<ResourceAncestors> {
        match resource {
            ResourceRef::Project(id) => {
                let row = sqlx::query!(
                    "SELECT organization_id FROM projects WHERE id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .to_domain()?;
                Ok(ResourceAncestors {
                    organization: row.map(|r| OrganizationId::new(r.organization_id)),
                    ..Default::default()
                })
            }
            ResourceRef::Pipeline(id) => {
                // `!`: sqlx cannot prove the inner join yields a row; both FKs are NOT NULL.
                let row = sqlx::query!(
                    "SELECT pl.project_id AS \"project_id!\", pr.organization_id AS \"organization_id!\" \
                     FROM pipelines pl JOIN projects pr ON pr.id = pl.project_id \
                     WHERE pl.id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .to_domain()?;
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
                .to_domain()?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(r.organization_id)),
                        project: Some(ProjectId::new(r.project_id)),
                        pipeline: Some(PipelineId::new(r.pipeline_id)),
                    }),
                    None => Ok(ResourceAncestors::default()),
                }
            }
            ResourceRef::Secret(id) => {
                let row = sqlx::query!(
                    "SELECT s.project_id AS \"project_id!\", pr.organization_id AS \"organization_id!\" \
                     FROM project_secrets s JOIN projects pr ON pr.id = s.project_id \
                     WHERE s.id = $1",
                    id.as_str(),
                )
                .fetch_optional(&self.pool)
                .await
                .to_domain()?;
                match row {
                    Some(r) => Ok(ResourceAncestors {
                        organization: Some(OrganizationId::new(r.organization_id)),
                        project: Some(ProjectId::new(r.project_id)),
                        pipeline: None,
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
                .to_domain()?;
                Ok(ResourceAncestors {
                    organization: row.map(|r| OrganizationId::new(r.organization_id)),
                    ..Default::default()
                })
            }
            _ => Ok(ResourceAncestors::default()),
        }
    }

    #[instrument(skip_all, fields(app_id = %app))]
    async fn app_is_active(&self, app: &AppId) -> DomainResult<bool> {
        let row = sqlx::query!("SELECT is_active FROM apps WHERE id = $1", app.as_str())
            .fetch_optional(&self.pool)
            .await
            .to_domain()?;
        // No row: the App was deleted; inactive, so an in-flight stream is denied.
        Ok(row.is_some_and(|r| r.is_active))
    }
}

#[cfg(test)]
mod tests;
