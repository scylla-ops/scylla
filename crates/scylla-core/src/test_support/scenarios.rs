//! Composite seeders that span multiple aggregates. Tests that just want a
//! parent chain so they can focus on the leaf aggregate use these.

#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "postgres")]
use crate::domain::entities::{Organization, Pipeline, Project};

/// Seed `org -> project -> pipeline` chain. Returns all three so a test can
/// reference any link without re-querying.
#[cfg(feature = "postgres")]
pub async fn seed_org_project_pipeline(
    pool: &PgPool,
    suffix: &str,
) -> (Organization, Project, Pipeline) {
    use super::organizations::seed_org;
    use super::pipelines::seed_pipeline;
    use super::projects::seed_project;

    let org = seed_org(pool, &format!("org-{suffix}")).await;
    let project = seed_project(pool, &org, &format!("project-{suffix}")).await;
    let pipeline = seed_pipeline(pool, &project).await;
    (org, project, pipeline)
}
