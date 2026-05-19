//! `Agent` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::entities::{Agent, AgentId};
use crate::domain::value_objects::agent::Hostname;

pub struct AgentBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl AgentBuilder {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn, into)] hostname: String,
        id: Option<AgentId>,
        last_seen_at: Option<DateTime<Utc>>,
        shutdown_at: Option<DateTime<Utc>>,
        #[builder(default = 5)] heartbeat_interval_secs: u64,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        /// Convenience: marks the agent as gracefully shut down (sets `shutdown_at = now`).
        #[builder(default = false)]
        shutdown: bool,
    ) -> Agent {
        let now = created_at.unwrap_or_else(Utc::now);
        let shutdown_at = if shutdown {
            Some(Utc::now())
        } else {
            shutdown_at
        };
        Agent::from_persistence(
            id.unwrap_or_else(AgentId::generate),
            Hostname::new(hostname).expect("test hostname invalid"),
            last_seen_at.unwrap_or(now),
            shutdown_at,
            heartbeat_interval_secs,
            now,
            updated_at.unwrap_or(now),
        )
    }
}

#[must_use]
pub fn agent(hostname: &str) -> Agent {
    AgentBuilder::new(hostname).build()
}

#[cfg(feature = "postgres")]
pub async fn seed_agent(pool: &sqlx::PgPool, hostname: &str) -> Agent {
    use crate::application::AgentRepository;
    use crate::infrastructure::persistence::postgres::PgAgentRepository;
    let agent = agent(hostname);
    PgAgentRepository::new(pool.clone())
        .create(&agent)
        .await
        .expect("seed agent failed");
    agent
}
