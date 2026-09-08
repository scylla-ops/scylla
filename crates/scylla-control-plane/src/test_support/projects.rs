//! `Project` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::clock;
use crate::domain::ids::{OrganizationId, ProjectId};
use crate::domain::organization::Organization;
use crate::domain::project::Project;
use crate::domain::project::{ProjectDescription, ProjectName};

pub struct ProjectBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl ProjectBuilder {
    /// Build a project belonging to `org`.
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn)] org: &Organization,
        #[builder(start_fn, into)] name: String,
        id: Option<ProjectId>,
        #[builder(into)] description: Option<String>,
        #[builder(default = true)] is_active: bool,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Project {
        Self::assemble_from_org_id(
            org.id().clone(),
            name,
            id,
            description,
            is_active,
            created_at,
            updated_at,
        )
    }

    /// Variant for FK-violation tests: targets an `OrganizationId` that may not exist.
    #[builder(start_fn = for_org_id, finish_fn = build)]
    pub fn assemble_from_org_id(
        #[builder(start_fn)] organization_id: OrganizationId,
        #[builder(start_fn, into)] name: String,
        id: Option<ProjectId>,
        #[builder(into)] description: Option<String>,
        #[builder(default = true)] is_active: bool,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Project {
        let now = created_at.unwrap_or_else(clock::now);
        let description =
            description.map(|s| ProjectDescription::new(s).expect("test project desc invalid"));
        Project::from_persistence(
            id.unwrap_or_else(ProjectId::generate),
            ProjectName::new(name).expect("test project name invalid"),
            description,
            organization_id,
            is_active,
            now,
            updated_at.unwrap_or(now),
        )
    }
}

#[must_use]
pub fn project(org: &Organization, name: &str) -> Project {
    ProjectBuilder::new(org, name).build()
}

pub async fn seed_project(pool: &sqlx::PgPool, org: &Organization, name: &str) -> Project {
    use crate::application::ProjectRepository;
    use crate::infrastructure::persistence::postgres::PgProjectRepository;
    let project = project(org, name);
    PgProjectRepository::new(pool.clone())
        .create(&project)
        .await
        .expect("seed project failed");
    project
}
