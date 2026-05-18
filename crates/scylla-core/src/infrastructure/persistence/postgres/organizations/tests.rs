use super::PgOrganizationRepository;
use crate::application::ports::OrganizationRepository;
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

#[sqlx::test(migrations = "../../migrations")]
async fn update_changes_description_to_none(pool: PgPool) {
    let repo = PgOrganizationRepository::new(pool);
    let mut org = OrgBuilder::new("Pied Piper")
        .description("starts with one")
        .build();
    repo.create(&org).await.expect("create");

    org.update_description(None).unwrap();
    repo.update(&org).await.expect("update");

    assert!(repo.find_by_id(org.id()).await.unwrap().description().is_none());
}
