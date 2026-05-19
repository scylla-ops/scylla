use super::PgUserProjectRepository;
use crate::application::{ProjectRepository, UserProjectRepository};
use crate::infrastructure::persistence::postgres::PgProjectRepository;
use crate::test_support::prelude::*;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn add_member_is_idempotent(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let project = seed_project(&pool, &org, "rocket").await;
    let repo = PgUserProjectRepository::new(pool);

    repo.add_member(user.id(), project.id()).await.unwrap();
    repo.add_member(user.id(), project.id()).await.unwrap();

    assert_eq!(
        repo.list_members(project.id(), None)
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
    let project = seed_project(&pool, &org, "rocket").await;
    let repo = PgUserProjectRepository::new(pool);

    assert!(!repo.is_member(user.id(), project.id()).await.unwrap());
    repo.add_member(user.id(), project.id()).await.unwrap();
    assert!(repo.is_member(user.id(), project.id()).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn remove_member_idempotent(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let project = seed_project(&pool, &org, "rocket").await;
    let repo = PgUserProjectRepository::new(pool);

    repo.remove_member(user.id(), project.id()).await.unwrap();
    repo.add_member(user.id(), project.id()).await.unwrap();
    repo.remove_member(user.id(), project.id()).await.unwrap();
    assert!(!repo.is_member(user.id(), project.id()).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_project_delete_removes_membership(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let org = seed_org(&pool, "acme").await;
    let project = seed_project(&pool, &org, "rocket").await;
    let repo = PgUserProjectRepository::new(pool.clone());
    repo.add_member(user.id(), project.id()).await.unwrap();

    PgProjectRepository::new(pool)
        .delete(project.id())
        .await
        .unwrap();

    assert_eq!(
        repo.list_user_projects(user.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        0,
    );
}
