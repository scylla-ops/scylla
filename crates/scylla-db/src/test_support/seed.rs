//! `seed_*` helpers: persist a fixture through the real Postgres repository.
//!
//! The in-memory builders they use (`org(..)`, `project(..)`, ...) live in
//! `scylla_core::test_support`; these functions only add the round trip through
//! the adapter, so a test can start from rows that exist.

use crate::domain::ids::JobId;
use crate::domain::ids::UserId;
use crate::domain::job::Job;
use crate::domain::job::JobLog;
use crate::domain::organization::Organization;
use crate::domain::pipeline::Pipeline;
use crate::domain::project::Project;
use crate::domain::session::Session;
use crate::domain::user::User;
use scylla_core::test_support::prelude::*;

pub async fn seed_org(pool: &sqlx::PgPool, name: &str) -> Organization {
    use crate::application::OrganizationRepository;
    use crate::postgres::PgOrganizationRepository;
    let org = org(name);
    PgOrganizationRepository::new(pool.clone())
        .create(&org)
        .await
        .expect("seed org failed");
    org
}

pub async fn seed_project(pool: &sqlx::PgPool, org: &Organization, name: &str) -> Project {
    use crate::application::ProjectRepository;
    use crate::postgres::PgProjectRepository;
    let project = project(org, name);
    PgProjectRepository::new(pool.clone())
        .create(&project)
        .await
        .expect("seed project failed");
    project
}

pub async fn seed_pipeline(pool: &sqlx::PgPool, project: &Project) -> Pipeline {
    use crate::application::PipelineRepository;
    use crate::postgres::PgPipelineRepository;
    let pipeline = pipeline(project);
    PgPipelineRepository::new(pool.clone())
        .create(&pipeline)
        .await
        .expect("seed pipeline failed");
    pipeline
}

pub async fn seed_job(pool: &sqlx::PgPool, pipeline: &Pipeline) -> Job {
    use crate::application::JobRepository;
    use crate::postgres::PgJobRepository;
    let job = job(pipeline);
    PgJobRepository::new(pool.clone())
        .create(&job)
        .await
        .expect("seed job failed");
    job
}

pub async fn seed_job_log(
    pool: &sqlx::PgPool,
    job_id: &JobId,
    node_id: &str,
    line: &str,
) -> JobLog {
    use crate::application::JobLogRepository;
    use crate::postgres::PgJobLogRepository;
    let log = job_log(job_id, node_id, line);
    PgJobLogRepository::new(pool.clone())
        .create(&log)
        .await
        .expect("seed job log failed");
    log
}

pub async fn seed_session(pool: &sqlx::PgPool, user_id: &UserId) -> Session {
    use crate::application::SessionRepository;
    use crate::postgres::PgSessionRepository;
    let session = session(user_id);
    PgSessionRepository::new(pool.clone())
        .create(&session)
        .await
        .expect("seed session failed");
    session
}

pub async fn seed_user(pool: &sqlx::PgPool, name: &str) -> User {
    use crate::application::UserRepository;
    use crate::postgres::PgUserRepository;
    let user = user(name);
    PgUserRepository::new(pool.clone())
        .create(&user)
        .await
        .expect("seed user failed");
    user
}
