use super::PgSessionRepository;
use crate::application::ports::{SessionRepository, UserRepository};
use crate::domain::errors::DomainError;
use crate::infrastructure::persistence::postgres::PgUserRepository;
use crate::test_support::prelude::*;
use chrono::Duration;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_find_by_token(pool: PgPool) {
    let user = seed_user(&pool, "alice").await;
    let repo = PgSessionRepository::new(pool);
    let session = SessionBuilder::new(user.id()).build();

    repo.create(&session).await.expect("create");
    let found = repo.find_by_token(session.token()).await.expect("find");

    assert_eq!(found.id(), session.id());
    assert_eq!(found.user_id(), user.id());
    assert_eq!(found.created_at(), session.created_at());
    assert_eq!(found.expires_at(), session.expires_at());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_token_not_found(pool: PgPool) {
    let repo = PgSessionRepository::new(pool);
    let res = repo.find_by_token("does-not-exist").await;
    assert!(matches!(res, Err(DomainError::NotFound { .. })));
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_extends_expiration(pool: PgPool) {
    let user = seed_user(&pool, "carol").await;
    let repo = PgSessionRepository::new(pool);
    let mut session = SessionBuilder::new(user.id()).build();
    repo.create(&session).await.expect("create");

    let original = session.expires_at();
    session.extend(Duration::hours(24));
    repo.update(&session).await.expect("update");

    let found = repo.find_by_token(session.token()).await.expect("find");
    assert!(found.expires_at() > original);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_expired_removes_only_past_sessions(pool: PgPool) {
    let user = seed_user(&pool, "expired-dave").await;
    let repo = PgSessionRepository::new(pool);

    let fresh = SessionBuilder::new(user.id()).build();
    repo.create(&fresh).await.expect("seed fresh");

    let expired = SessionBuilder::new(user.id()).expired(true).build();
    repo.create(&expired).await.expect("seed expired");

    let removed = repo.delete_expired().await.expect("sweep");
    assert_eq!(removed, 1);
    assert!(repo.find_by_token(fresh.token()).await.is_ok());
    assert!(repo.find_by_token(expired.token()).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_user_delete_removes_sessions(pool: PgPool) {
    let user = seed_user(&pool, "evictee").await;
    let session_repo = PgSessionRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool);

    let session = SessionBuilder::new(user.id()).build();
    session_repo.create(&session).await.expect("create");

    user_repo.delete(user.id()).await.expect("delete user");

    assert!(matches!(
        session_repo.find_by_token(session.token()).await,
        Err(DomainError::NotFound { .. })
    ));
}
