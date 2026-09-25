//! Order: `hooks` before the core builds its services, `prepare` before anything serves, `install` after the core's services exist.

use crate::surface::Surface;
use scylla_auth::authz::{PermissionService, PolicyControl, VisibilityResolver};
use scylla_extension::{Actions, Hooks};
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type PrepareFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// A feature authorizes through `actions`, as the core does. `permissions` stays until the
/// Enterprise features move to `actions`; `policy_control` and `visibility` serve a `Fetch` scope.
#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
    pub actions: Arc<Actions>,
    pub permissions: Arc<dyn PermissionService>,
    pub policy_control: Arc<dyn PolicyControl>,
    pub visibility: Arc<dyn VisibilityResolver>,
}

pub trait Feature: Send + 'static {
    fn hooks(&self, _hooks: &mut Hooks) {}

    fn prepare<'a>(&'a self, _db: &'a PgPool) -> PrepareFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn install(self: Box<Self>, _ctx: &Context, _surface: &mut Surface) {}
}
