use crate::application::agent::dispatch::JobDispatch;
use crate::application::agent::dispatch_port::AgentDispatch;
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, PipelineId};
use crate::domain::permission::Permission;
use scylla_auth::authz::PermissionService;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{instrument, warn};

pub enum DispatchOutcome {
    Dispatched(AppId),
    NoAgentAvailable,
}

pub struct DispatchUseCases<W: AgentDispatch, PS: PermissionService> {
    registry: Arc<W>,
    permission_service: Arc<PS>,
    next: AtomicUsize,
}

impl<W: AgentDispatch, PS: PermissionService> DispatchUseCases<W, PS> {
    #[must_use]
    pub fn new(registry: Arc<W>, permission_service: Arc<PS>) -> Self {
        Self {
            registry,
            permission_service,
            next: AtomicUsize::new(0),
        }
    }

    #[instrument(skip_all, fields(pipeline_id = %pipeline_id, job_id = %dispatch.job_id))]
    pub async fn dispatch_job(
        &self,
        pipeline_id: &PipelineId,
        dispatch: &JobDispatch,
    ) -> DomainResult<DispatchOutcome> {
        let agents = self.registry.connected();
        if agents.is_empty() {
            warn!(pipeline_id = %pipeline_id, "no connected agent; job left pending");
            return Ok(DispatchOutcome::NoAgentAvailable);
        }

        // Idlest first, then rotate among equals so eligible agents take turns.
        let start = self.next.fetch_add(1, Ordering::Relaxed);
        let n = agents.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| {
            (
                self.registry.in_flight(&agents[i]),
                start.wrapping_add(i) % n,
            )
        });
        for i in order {
            let app_id = &agents[i];
            let caller = CallerContext::App(app_id.clone());
            match self
                .permission_service
                .check(&caller, Permission::ExecuteJob(pipeline_id.clone()))
                .await
            {
                Ok(()) => match self.registry.dispatch(app_id, dispatch).await {
                    Ok(()) => return Ok(DispatchOutcome::Dispatched(app_id.clone())),
                    // Disconnected since `connected()` was snapshotted: try the next one.
                    Err(e) => {
                        warn!(app_id = %app_id, error = %e, "dispatch to agent failed; trying next");
                    }
                },
                Err(DomainError::Forbidden(_)) => {}
                Err(e) => {
                    warn!(app_id = %app_id, error = %e, "authz check errored during dispatch; skipping agent");
                }
            }
        }
        warn!(
            pipeline_id = %pipeline_id,
            "no connected agent authorized to execute pipeline; job left pending"
        );
        Ok(DispatchOutcome::NoAgentAvailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubRegistry {
        connected: Vec<AppId>,
        dispatched: Mutex<Vec<String>>,
        loads: Mutex<HashMap<String, usize>>,
    }

    impl StubRegistry {
        fn new(connected: Vec<AppId>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(HashMap::new()),
            }
        }
        fn with_loads(connected: Vec<AppId>, loads: HashMap<String, usize>) -> Self {
            Self {
                connected,
                dispatched: Mutex::new(vec![]),
                loads: Mutex::new(loads),
            }
        }
    }

    #[async_trait]
    impl AgentDispatch for StubRegistry {
        fn connected(&self) -> Vec<AppId> {
            self.connected.clone()
        }
        async fn dispatch(&self, app_id: &AppId, _dispatch: &JobDispatch) -> DomainResult<()> {
            self.dispatched
                .lock()
                .unwrap()
                .push(app_id.as_str().to_string());
            *self
                .loads
                .lock()
                .unwrap()
                .entry(app_id.as_str().to_string())
                .or_insert(0) += 1;
            Ok(())
        }
        fn disconnect(&self, _app_id: &AppId) {}
        fn in_flight(&self, app_id: &AppId) -> usize {
            *self
                .loads
                .lock()
                .unwrap()
                .get(app_id.as_str())
                .unwrap_or(&0)
        }
        fn release(&self, app_id: &AppId) {
            if let Some(v) = self.loads.lock().unwrap().get_mut(app_id.as_str()) {
                *v = v.saturating_sub(1);
            }
        }
    }

    struct StubPerms {
        allowed: &'static str,
    }

    #[async_trait]
    impl PermissionService for StubPerms {
        async fn check(&self, caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            if matches!(caller, CallerContext::App(id) if id.as_str() == self.allowed) {
                Ok(())
            } else {
                Err(DomainError::forbidden("not the allowed agent"))
            }
        }
    }

    struct StubPermsAll;

    #[async_trait]
    impl PermissionService for StubPermsAll {
        async fn check(&self, _caller: &CallerContext, _perm: Permission) -> DomainResult<()> {
            Ok(())
        }
    }

    fn dispatch() -> JobDispatch {
        JobDispatch {
            job_id: "j1".to_string(),
            pipeline_id: "pl1".to_string(),
            nodes: vec![],
        }
    }

    #[tokio::test]
    async fn dispatches_to_first_authorized_connected_agent() {
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-unauthorized"),
            AppId::new("app-ok"),
        ]));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPerms { allowed: "app-ok" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::Dispatched(id) if id.as_str() == "app-ok"));
        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-ok"]);
    }

    #[tokio::test]
    async fn spreads_jobs_round_robin_across_authorized_agents() {
        let registry = Arc::new(StubRegistry::new(vec![
            AppId::new("app-a"),
            AppId::new("app-b"),
        ]));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPermsAll));

        for _ in 0..4 {
            uc.dispatch_job(&PipelineId::new("pl1"), &dispatch())
                .await
                .unwrap();
        }

        let dispatched = registry.dispatched.lock().unwrap().clone();
        assert_eq!(dispatched.len(), 4);
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-a").count(),
            2,
            "app-a should get half the jobs"
        );
        assert_eq!(
            dispatched.iter().filter(|x| x.as_str() == "app-b").count(),
            2,
            "app-b should get half the jobs (no longer starved)"
        );
    }

    #[tokio::test]
    async fn picks_least_loaded_eligible_agent() {
        let mut loads = HashMap::new();
        loads.insert("app-busy".to_string(), 3);
        let registry = Arc::new(StubRegistry::with_loads(
            vec![AppId::new("app-busy"), AppId::new("app-idle")],
            loads,
        ));
        let uc = DispatchUseCases::new(registry.clone(), Arc::new(StubPermsAll));

        uc.dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert_eq!(registry.dispatched.lock().unwrap().as_slice(), ["app-idle"]);
    }

    #[tokio::test]
    async fn no_agent_when_none_authorized() {
        let registry = Arc::new(StubRegistry::new(vec![AppId::new("app-x")]));
        let uc = DispatchUseCases::new(registry, Arc::new(StubPerms { allowed: "nobody" }));

        let outcome = uc
            .dispatch_job(&PipelineId::new("pl1"), &dispatch())
            .await
            .unwrap();

        assert!(matches!(outcome, DispatchOutcome::NoAgentAvailable));
    }
}
