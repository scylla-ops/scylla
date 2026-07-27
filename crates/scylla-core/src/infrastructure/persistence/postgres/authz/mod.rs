use crate::application::authz::entity_provider::{AuthzEntityProvider, ResourceAncestors};
use crate::domain::entities::{AppId, OrganizationId, PipelineId, ProjectId};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::ResourceRef;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use super::error::SqlxResultExt;

/// Resolves a resource's place in the tenancy tree for Cedar, over the
/// pipeline→project→org foreign keys. Read-only. Principals need nothing
/// materialised: their authority lives in the grants compiled into the policy
/// set, so there is no per-principal query on the check path at all.
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
                .to_domain()?;
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
            // System / User / Organization have no tenancy parents.
            _ => Ok(ResourceAncestors::default()),
        }
    }

    #[instrument(skip(self), fields(app_id = %app))]
    async fn app_is_active(&self, app: &AppId) -> DomainResult<bool> {
        let row = sqlx::query!("SELECT is_active FROM apps WHERE id = $1", app.as_str())
            .fetch_optional(&self.pool)
            .await
            .to_domain()?;
        // No row → the App was deleted; treat as inactive so an in-flight stream
        // from a now-removed App is denied rather than trusted.
        Ok(row.is_some_and(|r| r.is_active))
    }
}
