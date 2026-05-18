use super::PgAgentRepository;
use crate::application::ports::AgentRepository;
use crate::domain::value_objects::agent::Hostname;
use crate::test_support::prelude::*;
use chrono::Utc;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn round_trip_with_no_shutdown(pool: PgPool) {
    let repo = PgAgentRepository::new(pool);
    let agent = agent("worker-1");

    repo.create(&agent).await.unwrap();
    let found = repo.find_by_id(agent.id()).await.unwrap();

    assert_eq!(found.id(), agent.id());
    assert_eq!(found.hostname().as_str(), "worker-1");
    assert!(found.shutdown_at().is_none());
    assert_eq!(found.heartbeat_interval_secs(), agent.heartbeat_interval_secs());
    assert_eq!(found.last_seen_at(), agent.last_seen_at());
}

#[sqlx::test(migrations = "../../migrations")]
async fn round_trip_with_shutdown_marker(pool: PgPool) {
    let repo = PgAgentRepository::new(pool);
    let agent = AgentBuilder::new("worker-2").shutdown(true).build();
    repo.create(&agent).await.unwrap();

    let found = repo.find_by_id(agent.id()).await.unwrap();
    assert!(found.shutdown_at().is_some());
    assert!(!found.is_connected());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_preserves_other_fields(pool: PgPool) {
    let repo = PgAgentRepository::new(pool);
    let mut agent = agent("worker-3");
    repo.create(&agent).await.unwrap();
    let original_created = agent.created_at();

    agent.record_heartbeat(Hostname::new("worker-3-renamed").unwrap(), 10);
    repo.update(&agent).await.unwrap();

    let found = repo.find_by_id(agent.id()).await.unwrap();
    assert_eq!(found.hostname().as_str(), "worker-3-renamed");
    assert_eq!(found.heartbeat_interval_secs(), 10);
    // created_at must NOT change on update.
    assert_eq!(found.created_at(), original_created);
    assert!(found.updated_at() <= Utc::now());
}

#[sqlx::test(migrations = "../../migrations")]
async fn exists_reflects_state(pool: PgPool) {
    let repo = PgAgentRepository::new(pool);
    let agent = agent("ghost");
    assert!(!repo.exists(agent.id()).await.unwrap());
    repo.create(&agent).await.unwrap();
    assert!(repo.exists(agent.id()).await.unwrap());
}
