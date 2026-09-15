use sqlx::PgPool;

use crate::domain::organization::Organization;
use crate::domain::pipeline::Pipeline;
use crate::domain::project::Project;

pub async fn seed_org_project_pipeline(
    pool: &PgPool,
    suffix: &str,
) -> (Organization, Project, Pipeline) {
    use super::seed::{seed_org, seed_pipeline, seed_project};

    let org = seed_org(pool, &format!("org-{suffix}")).await;
    let project = seed_project(pool, &org, &format!("project-{suffix}")).await;
    let pipeline = seed_pipeline(pool, &project).await;
    (org, project, pipeline)
}
