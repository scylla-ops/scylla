use crate::application::agent::dispatch::JobDispatch;
use crate::domain::entities::AppId;
use crate::domain::errors::DomainResult;
use async_trait::async_trait;

/// Registry of currently-connected agent Apps. Presence is the open agent
/// stream itself; the in-memory adapter holds one channel sender per connection.
/// Replaces the message broker for job dispatch (mono-instance).
#[async_trait]
pub trait AgentDispatch: Send + Sync {
    /// App ids that have a live agent stream right now.
    fn connected(&self) -> Vec<AppId>;

    /// Push a job dispatch to a connected app. Errors if the app is not
    /// currently connected (or its stream has gone away).
    async fn dispatch(&self, app_id: &AppId, dispatch: &JobDispatch) -> DomainResult<()>;

    /// Force-disconnect an agent by closing its stream. Called when an app's
    /// authorization changes (e.g. a grant is revoked) so a no-longer-authorized
    /// agent stops immediately instead of finishing its job. No-op if the app
    /// is not connected.
    fn disconnect(&self, app_id: &AppId);

    /// Jobs dispatched to `app_id` that have not yet reported a terminal status
    /// — the agent's current load. `0` for an unknown/disconnected app. Drives
    /// least-loaded selection so jobs land on the idlest eligible agent.
    fn in_flight(&self, app_id: &AppId) -> usize;

    /// Mark one job on `app_id` as settled (terminal status reported), freeing a
    /// load slot. Saturating — never underflows, so a stray release is harmless.
    fn release(&self, app_id: &AppId);
}
