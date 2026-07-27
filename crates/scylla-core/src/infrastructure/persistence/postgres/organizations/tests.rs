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

/// Removing a member must also delete the grants they hold scoped to the org,
/// so a later re-add starts from a clean slate instead of silently restoring
/// their old authority. The co-owner's grant and membership must be left
/// untouched.
#[sqlx::test(migrations = "../../migrations")]
async fn remove_member_and_grants_revokes_only_the_members_scoped_grants(pool: PgPool) {
    use crate::application::UserOrganizationRepository;
    use crate::application::authz::grant::{
        Grant, GrantRepository, ORGANIZATION_ADMIN_ROLE, Principal, Scope,
    };
    use crate::domain::value_objects::role::name::RoleName;
    use crate::infrastructure::persistence::postgres::{
        PgGrantRepository, PgUserOrganizationRepository,
    };

    let org = seed_org(&pool, "acme").await;
    let owner = seed_user(&pool, "owner").await;
    let victim = seed_user(&pool, "victim").await;

    let members = PgUserOrganizationRepository::new(pool.clone());
    members.add_member(owner.id(), org.id()).await.unwrap();
    members.add_member(victim.id(), org.id()).await.unwrap();

    let grants = PgGrantRepository::new(pool.clone());
    let admin_role = || RoleName::new(ORGANIZATION_ADMIN_ROLE).unwrap();
    let owner_grant = Grant::new(
        Principal::User(owner.id().clone()),
        admin_role(),
        Scope::Organization(org.id().clone()),
    );
    let victim_grant = Grant::new(
        Principal::User(victim.id().clone()),
        admin_role(),
        Scope::Organization(org.id().clone()),
    );
    grants.create(&owner_grant).await.unwrap();
    grants.create(&victim_grant).await.unwrap();

    PgOrganizationRepository::new(pool.clone())
        .remove_member_and_grants(victim.id(), org.id())
        .await
        .expect("remove");

    assert!(
        !members.is_member(victim.id(), org.id()).await.unwrap(),
        "victim membership must be gone",
    );
    assert!(
        members.is_member(owner.id(), org.id()).await.unwrap(),
        "owner membership must remain",
    );
    let remaining = grants.list_all().await.unwrap();
    assert!(
        remaining.iter().all(|g| g.id != victim_grant.id),
        "victim's grant must be deleted with the membership",
    );
    assert!(
        remaining.iter().any(|g| g.id == owner_grant.id),
        "the co-owner's grant must be untouched",
    );
}

/// Removing a member from an org strips everything they held *under* it too:
/// grants scoped to the org's projects and their memberships in those projects.
/// Identical holdings in another organization must survive — the purge is
/// scoped to the one org the user is leaving.
#[sqlx::test(migrations = "../../migrations")]
async fn remove_member_and_grants_purges_the_whole_org_subtree(pool: PgPool) {
    use crate::application::authz::grant::{
        Grant, GrantRepository, PROJECT_ADMIN_ROLE, PROJECT_AGENT_ROLE, Principal, Scope,
    };
    use crate::application::{UserOrganizationRepository, UserProjectRepository};
    use crate::domain::value_objects::role::name::RoleName;
    use crate::infrastructure::persistence::postgres::{
        PgGrantRepository, PgUserOrganizationRepository, PgUserProjectRepository,
    };

    let org = seed_org(&pool, "acme").await;
    let other_org = seed_org(&pool, "globex").await;
    let project = seed_project(&pool, &org, "apollo").await;
    let other_project = seed_project(&pool, &other_org, "zeus").await;
    let victim = seed_user(&pool, "victim").await;

    let org_members = PgUserOrganizationRepository::new(pool.clone());
    org_members.add_member(victim.id(), org.id()).await.unwrap();
    org_members
        .add_member(victim.id(), other_org.id())
        .await
        .unwrap();
    let project_members = PgUserProjectRepository::new(pool.clone());
    project_members
        .add_member(victim.id(), project.id())
        .await
        .unwrap();
    project_members
        .add_member(victim.id(), other_project.id())
        .await
        .unwrap();

    let grants = PgGrantRepository::new(pool.clone());
    let admin_role = || RoleName::new(PROJECT_ADMIN_ROLE).unwrap();
    let child_role_grant = Grant::new(
        Principal::User(victim.id().clone()),
        admin_role(),
        Scope::Project(project.id().clone()),
    );
    let child_agent_grant = Grant::new(
        Principal::User(victim.id().clone()),
        RoleName::new(PROJECT_AGENT_ROLE).unwrap(),
        Scope::Project(project.id().clone()),
    );
    let other_org_grant = Grant::new(
        Principal::User(victim.id().clone()),
        admin_role(),
        Scope::Project(other_project.id().clone()),
    );
    grants.create(&child_role_grant).await.unwrap();
    grants.create(&child_agent_grant).await.unwrap();
    grants.create(&other_org_grant).await.unwrap();

    PgOrganizationRepository::new(pool.clone())
        .remove_member_and_grants(victim.id(), org.id())
        .await
        .expect("remove");

    assert!(
        !project_members
            .is_member(victim.id(), project.id())
            .await
            .unwrap(),
        "membership in the org's project must be gone",
    );
    assert!(
        project_members
            .is_member(victim.id(), other_project.id())
            .await
            .unwrap(),
        "membership in the other org's project must remain",
    );
    let remaining = grants.list_all().await.unwrap();
    assert!(
        remaining
            .iter()
            .all(|g| g.id != child_role_grant.id && g.id != child_agent_grant.id),
        "grants on the org's projects must be deleted with the org membership",
    );
    assert!(
        remaining.iter().any(|g| g.id == other_org_grant.id),
        "grants under the other org must be untouched",
    );
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
