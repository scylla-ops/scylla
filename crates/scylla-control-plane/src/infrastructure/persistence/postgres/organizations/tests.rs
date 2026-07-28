use super::PgOrganizationRepository;
use crate::application::OrganizationRepository;
use crate::domain::value_objects::organization::OrganizationDescription;
use crate::test_support::prelude::*;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn round_trip_with_none_description(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let org = org("Acme");

    repo.create(&org).await.expect("create");
    let found = repo.find_by_id(org.id()).await.expect("find");

    assert_eq!(found.id(), org.id());
    assert!(found.description().is_none());
    assert_eq!(found.created_at(), org.created_at());
}

#[sqlx::test(migrations = "../../migrations")]
async fn round_trip_with_some_description(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let org = OrgBuilder::new("Globex")
        .description("Worldwide subsidiary")
        .build();
    repo.create(&org).await.expect("create");

    let found = repo.find_by_id(org.id()).await.expect("find");
    assert_eq!(
        found.description().map(OrganizationDescription::as_str),
        Some("Worldwide subsidiary"),
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_name(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let org = org("Initech");
    repo.create(&org).await.expect("create");

    let found = repo.find_by_name(org.name()).await.expect("find");
    assert_eq!(found.id(), org.id());
}

#[sqlx::test(migrations = "../../migrations")]
async fn name_exists_reflects_state(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let org = org("Hooli");

    assert!(!repo.name_exists(org.name()).await.unwrap());
    repo.create(&org).await.expect("create");
    assert!(repo.name_exists(org.name()).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_active_filters_inactive(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    repo.create(&org("Active1")).await.expect("seed");
    repo.create(&org("Active2")).await.expect("seed");
    repo.create(&OrgBuilder::new("Dormant").is_active(false).build())
        .await
        .expect("seed inactive");

    let active = repo.list_active(None).await.expect("list");
    assert_eq!(active.metadata().total_count(), 2);
    let all = repo.list_all(None).await.expect("list");
    assert_eq!(all.metadata().total_count(), 3);
}

/// The kill switch. Stripping someone's access to an organization has to reach
/// every project underneath it in one statement, and must leave both the other
/// people and the same person's holdings in other organizations untouched.
///
/// This is the anti-regression for the leak that used to be covered by the Cedar
/// membership guard: with membership gone, deleting the rows *is* the boundary,
/// so it has to be complete.
#[sqlx::test(migrations = "../../migrations")]
async fn revoke_all_access_strips_the_whole_org_subtree(pool: PgPool) {
    use crate::application::authz::grant::{
        Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, PROJECT_ADMIN_ROLE, PROJECT_AGENT_ROLE,
        Principal, SYSTEM_ADMIN_ROLE, Scope,
    };
    use crate::domain::value_objects::role::name::RoleName;
    use crate::infrastructure::persistence::postgres::PgGrantRepository;

    let org = seed_org(&pool, "acme").await;
    let other_org = seed_org(&pool, "globex").await;
    let project = seed_project(&pool, &org, "apollo").await;
    let other_project = seed_project(&pool, &other_org, "zeus").await;
    let victim = seed_user(&pool, "victim").await;
    let colleague = seed_user(&pool, "colleague").await;

    let grants = PgGrantRepository::new(pool.clone());
    let role = |name: &str| RoleName::new(name).unwrap();
    let victim_principal = Principal::User(victim.id().clone());

    let doomed = [
        Grant::new(
            victim_principal.clone(),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(org.id().clone()),
        ),
        Grant::new(
            victim_principal.clone(),
            role(PROJECT_ADMIN_ROLE),
            Scope::Project(project.id().clone()),
        ),
        Grant::new(
            victim_principal.clone(),
            role(PROJECT_AGENT_ROLE),
            Scope::Project(project.id().clone()),
        ),
    ];
    let survivors = [
        // Same person, another organization.
        Grant::new(
            victim_principal.clone(),
            role(PROJECT_ADMIN_ROLE),
            Scope::Project(other_project.id().clone()),
        ),
        // A platform operator's global access, which an org-level revoke must
        // never be able to strip.
        Grant::new(
            victim_principal.clone(),
            role(SYSTEM_ADMIN_ROLE),
            Scope::System,
        ),
        // Someone else, so the org keeps an owner and the revoke is allowed.
        Grant::new(
            Principal::User(colleague.id().clone()),
            role(ORGANIZATION_ADMIN_ROLE),
            Scope::Organization(org.id().clone()),
        ),
    ];
    for g in doomed.iter().chain(survivors.iter()) {
        grants.create(g).await.unwrap();
    }

    let removed = grants
        .revoke_all(&victim_principal, &Scope::Organization(org.id().clone()))
        .await
        .expect("revoke all");
    assert_eq!(removed, 3, "the org grant plus both project grants");

    let remaining = grants.list_all().await.unwrap();
    for g in &doomed {
        assert!(
            !remaining.iter().any(|r| r.id == g.id),
            "grant {} should have gone with the organization access",
            g.id,
        );
    }
    for g in &survivors {
        assert!(
            remaining.iter().any(|r| r.id == g.id),
            "grant {} is outside the revoked scope and must survive",
            g.id,
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_changes_description_to_none(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let mut org = OrgBuilder::new("Pied Piper")
        .description("starts with one")
        .build();
    repo.create(&org).await.expect("create");

    org.update_description(None).unwrap();
    repo.update(&org).await.expect("update");

    assert!(
        repo.find_by_id(org.id())
            .await
            .unwrap()
            .description()
            .is_none()
    );
}
