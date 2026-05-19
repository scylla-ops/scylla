use super::PgUserOrganizationRepository;
use crate::application::{UserOrganizationRepository, UserRepository};
use crate::infrastructure::persistence::postgres::PgUserRepository;
use crate::test_support::prelude::*;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn add_member_is_idempotent(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let repo = PgUserOrganizationRepository::new(pool);

    repo.add_member(user.id(), org.id()).await.unwrap();
    repo.add_member(user.id(), org.id()).await.unwrap(); // no error on dup

    assert_eq!(
        repo.list_members(org.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        1,
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn is_member_reflects_membership(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let repo = PgUserOrganizationRepository::new(pool);

    assert!(!repo.is_member(user.id(), org.id()).await.unwrap());
    repo.add_member(user.id(), org.id()).await.unwrap();
    assert!(repo.is_member(user.id(), org.id()).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn remove_member_idempotent(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let repo = PgUserOrganizationRepository::new(pool);

    // Remove without prior add must not panic.
    repo.remove_member(user.id(), org.id()).await.unwrap();

    repo.add_member(user.id(), org.id()).await.unwrap();
    repo.remove_member(user.id(), org.id()).await.unwrap();
    assert!(!repo.is_member(user.id(), org.id()).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_user_delete_removes_membership(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let repo = PgUserOrganizationRepository::new(pool.clone());
    repo.add_member(user.id(), org.id()).await.unwrap();

    PgUserRepository::new(pool).delete(user.id()).await.unwrap();

    assert_eq!(
        repo.list_members(org.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        0,
    );
}
