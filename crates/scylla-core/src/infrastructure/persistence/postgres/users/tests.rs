use super::PgUserRepository;
use crate::application::UserRepository;
use crate::domain::entities::UserId;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::PaginationParams;
use crate::domain::value_objects::user::Username;
use crate::test_support::prelude::*;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_find_round_trips(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = user("alice");

    repo.create(&user).await.expect("create");
    let found = repo.find_by_id(user.id()).await.expect("find");

    assert_eq!(found.id(), user.id());
    assert_eq!(found.username(), user.username());
    assert_eq!(
        found.password_hash().as_str(),
        user.password_hash().as_str()
    );
    assert_eq!(found.is_active(), user.is_active());
    // chrono normalizes through TIMESTAMPTZ; equality proves UTC preservation.
    assert_eq!(found.created_at(), user.created_at());
    assert_eq!(found.updated_at(), user.updated_at());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_not_found_returns_not_found_error(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let res = repo.find_by_id(&UserId::generate()).await;
    assert!(matches!(res, Err(DomainError::NotFound { .. })));
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_username_returns_persisted_user(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = user("bob");
    repo.create(&user).await.expect("create");

    let found = repo.find_by_username(user.username()).await.expect("find");
    assert_eq!(found.id(), user.id());
}

#[sqlx::test(migrations = "../../migrations")]
async fn inactive_user_round_trips(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = UserBuilder::new("carol").is_active(false).build();
    repo.create(&user).await.expect("create");

    assert!(!repo.find_by_id(user.id()).await.unwrap().is_active());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_modifies_fields_then_visible_on_read(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let mut user = user("dave");
    repo.create(&user).await.expect("create");

    let new_name = Username::new("davinci").unwrap();
    user.update_username(new_name.clone()).unwrap();
    repo.update(&user).await.expect("update");

    assert_eq!(
        repo.find_by_id(user.id()).await.unwrap().username(),
        &new_name
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_row(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = user("eve");
    repo.create(&user).await.expect("create");

    repo.delete(user.id()).await.expect("delete");
    assert!(matches!(
        repo.find_by_id(user.id()).await,
        Err(DomainError::NotFound { .. })
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn unique_username_violation_maps_to_conflict(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    repo.create(&user("frank")).await.expect("first");

    let dup = user("frank"); // same username, different generated ULID
    assert!(matches!(
        repo.create(&dup).await,
        Err(DomainError::Conflict(_))
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_all_paginates_in_creation_order(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    for n in &["a", "b", "c"] {
        repo.create(&user(n)).await.expect("seed");
    }

    let page = PaginationParams::new(1, 20).unwrap();
    let result = repo.list_all(Some(&page)).await.expect("list");

    assert_eq!(result.metadata().total_count(), 3);
    assert_eq!(result.items().len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn username_exists_reflects_state(pool: PgPool) {
    let repo = PgUserRepository::new(pool);
    let user = user("grace");

    assert!(!repo.username_exists(user.username()).await.unwrap());
    repo.create(&user).await.expect("create");
    assert!(repo.username_exists(user.username()).await.unwrap());
}
