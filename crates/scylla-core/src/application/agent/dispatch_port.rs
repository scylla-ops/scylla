use crate::application::agent::dispatch::JobDispatch;
use crate::domain::errors::DomainResult;
use crate::domain::ids::AppId;
use async_trait::async_trait;

#[async_trait]
pub trait AgentDispatch: Send + Sync {
    fn connected(&self) -> Vec<AppId>;

    async fn dispatch(&self, app_id: &AppId, dispatch: &JobDispatch) -> DomainResult<()>;

    fn disconnect(&self, app_id: &AppId);

    fn in_flight(&self, app_id: &AppId) -> usize;

    fn release(&self, app_id: &AppId);
}
