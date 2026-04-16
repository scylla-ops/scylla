use crate::application::ports::AgentRepository;
use crate::domain::entities::{Agent, AgentId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::agent::Hostname;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct AgentUseCases<A: AgentRepository> {
    agent_repo: Arc<A>,
}

impl<A: AgentRepository> AgentUseCases<A> {
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn get(&self, id: &AgentId) -> DomainResult<Agent> {
        self.agent_repo.find_by_id(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Agent>> {
        self.agent_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn delete(&self, id: &AgentId) -> DomainResult<()> {
        self.agent_repo.find_by_id(id).await?;
        self.agent_repo.delete(id).await
    }

    /// Record a heartbeat. Creates the row on first sight, otherwise refreshes
    /// `last_seen_at` + `hostname` + `heartbeat_interval_secs`.
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn record_heartbeat(
        &self,
        id: &AgentId,
        hostname: Hostname,
        heartbeat_interval_secs: u64,
    ) -> DomainResult<Agent> {
        match self.agent_repo.find_by_id(id).await {
            Ok(mut agent) => {
                agent.record_heartbeat(hostname, heartbeat_interval_secs);
                self.agent_repo.update(&agent).await
            }
            Err(DomainError::NotFound { .. }) => {
                let agent = Agent::create(id.clone(), hostname, heartbeat_interval_secs);
                self.agent_repo.create(&agent).await
            }
            Err(e) => Err(e),
        }
    }

    /// Record a graceful shutdown. Stamps `shutdown_at` so derived status flips
    /// to disconnected on the next read; `last_seen_at` is preserved.
    #[instrument(skip(self), fields(agent_id = %id))]
    pub async fn record_shutdown(&self, id: &AgentId) -> DomainResult<Agent> {
        let mut agent = self.agent_repo.find_by_id(id).await?;
        agent.record_shutdown();
        self.agent_repo.update(&agent).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::PaginatedResult;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct StubAgentRepo {
        state: Mutex<Option<Agent>>,
        find_err: Mutex<Option<DomainError>>,
        created: Mutex<bool>,
        updated: Mutex<bool>,
    }

    impl StubAgentRepo {
        fn with_agent(agent: Agent) -> Self {
            Self {
                state: Mutex::new(Some(agent)),
                ..Self::default()
            }
        }

        fn with_find_err(err: DomainError) -> Self {
            Self {
                find_err: Mutex::new(Some(err)),
                ..Self::default()
            }
        }

        fn was_created(&self) -> bool {
            *self.created.lock().unwrap()
        }

        fn was_updated(&self) -> bool {
            *self.updated.lock().unwrap()
        }
    }

    #[async_trait]
    impl AgentRepository for StubAgentRepo {
        async fn create(&self, agent: &Agent) -> DomainResult<Agent> {
            *self.created.lock().unwrap() = true;
            *self.state.lock().unwrap() = Some(agent.clone());
            Ok(agent.clone())
        }

        async fn find_by_id(&self, id: &AgentId) -> DomainResult<Agent> {
            if let Some(err) = self.find_err.lock().unwrap().take() {
                return Err(err);
            }
            self.state.lock().unwrap().clone().ok_or_else(|| {
                DomainError::not_found("Agent", id.as_str())
            })
        }

        async fn update(&self, agent: &Agent) -> DomainResult<Agent> {
            *self.updated.lock().unwrap() = true;
            *self.state.lock().unwrap() = Some(agent.clone());
            Ok(agent.clone())
        }

        async fn delete(&self, _id: &AgentId) -> DomainResult<()> {
            *self.state.lock().unwrap() = None;
            Ok(())
        }

        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Agent>> {
            unimplemented!()
        }

        async fn exists(&self, _id: &AgentId) -> DomainResult<bool> {
            Ok(self.state.lock().unwrap().is_some())
        }
    }

    fn id() -> AgentId {
        AgentId::new("agent-1")
    }

    fn host() -> Hostname {
        Hostname::new("host").unwrap()
    }

    #[tokio::test]
    async fn heartbeat_creates_on_not_found() {
        let repo = Arc::new(StubAgentRepo::with_find_err(DomainError::not_found(
            "Agent", "agent-1",
        )));
        let uc = AgentUseCases::new(repo.clone());

        let agent = uc.record_heartbeat(&id(), host(), 5).await.unwrap();

        assert!(repo.was_created());
        assert!(!repo.was_updated());
        assert_eq!(agent.hostname().as_str(), "host");
    }

    #[tokio::test]
    async fn heartbeat_updates_existing() {
        let existing = Agent::create(id(), Hostname::new("old").unwrap(), 5);
        let repo = Arc::new(StubAgentRepo::with_agent(existing));
        let uc = AgentUseCases::new(repo.clone());

        let agent = uc
            .record_heartbeat(&id(), Hostname::new("new").unwrap(), 10)
            .await
            .unwrap();

        assert!(repo.was_updated());
        assert!(!repo.was_created());
        assert_eq!(agent.hostname().as_str(), "new");
        assert_eq!(agent.heartbeat_interval_secs(), 10);
    }

    #[tokio::test]
    async fn heartbeat_propagates_non_notfound_error() {
        let repo = Arc::new(StubAgentRepo::with_find_err(DomainError::infrastructure(
            "db down",
        )));
        let uc = AgentUseCases::new(repo.clone());

        let err = uc.record_heartbeat(&id(), host(), 5).await.unwrap_err();

        assert!(matches!(err, DomainError::Infrastructure(_)));
        assert!(!repo.was_created());
        assert!(!repo.was_updated());
    }

    #[tokio::test]
    async fn shutdown_disconnects_existing() {
        let existing = Agent::create(id(), host(), 5);
        let repo = Arc::new(StubAgentRepo::with_agent(existing));
        let uc = AgentUseCases::new(repo.clone());

        let agent = uc.record_shutdown(&id()).await.unwrap();

        assert!(repo.was_updated());
        assert!(!agent.is_connected());
        assert!(agent.shutdown_at().is_some());
    }

    #[tokio::test]
    async fn shutdown_errors_when_missing() {
        let repo = Arc::new(StubAgentRepo::with_find_err(DomainError::not_found(
            "Agent", "agent-1",
        )));
        let uc = AgentUseCases::new(repo);

        let err = uc.record_shutdown(&id()).await.unwrap_err();

        assert!(matches!(err, DomainError::NotFound { .. }));
    }
}
