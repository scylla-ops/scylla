use crate::domain::entities::AppId;
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::pipeline::JobDispatch;
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

    /// Force-disconnect a agent by closing its stream. Called when an app's
    /// authorization changes (e.g. a grant is revoked) so a no-longer-authorized
    /// agent stops immediately instead of finishing its job. No-op if the app
    /// is not connected.
    fn disconnect(&self, app_id: &AppId);
}
