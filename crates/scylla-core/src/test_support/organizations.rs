//! `Organization` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::clock;
use crate::domain::entities::{Organization, OrganizationId};
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};

pub struct OrgBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl OrgBuilder {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn, into)] name: String,
        id: Option<OrganizationId>,
        #[builder(into)] description: Option<String>,
        #[builder(default = true)] is_active: bool,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Organization {
        let now = created_at.unwrap_or_else(clock::now);
        let description =
            description.map(|s| OrganizationDescription::new(s).expect("test org desc invalid"));
        Organization::from_persistence(
            id.unwrap_or_else(OrganizationId::generate),
            OrganizationName::new(name).expect("test org name invalid"),
            description,
            is_active,
            now,
            updated_at.unwrap_or(now),
        )
    }
}

#[must_use]
pub fn org(name: &str) -> Organization {
    OrgBuilder::new(name).build()
}

#[cfg(feature = "postgres")]
pub async fn seed_org(pool: &sqlx::PgPool, name: &str) -> Organization {
    use crate::application::OrganizationRepository;
    use crate::infrastructure::persistence::postgres::PgOrganizationRepository;
    let org = org(name);
    PgOrganizationRepository::new(pool.clone())
        .create(&org)
        .await
        .expect("seed org failed");
    org
}
