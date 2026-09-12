//! Features: the unit an edition adds to the server.
//!
//! A feature is everything one capability needs, kept together: the extension
//! implementations the core consults, the migrations its own tables need, and
//! the services it puts on the listener. It reaches the server in three steps,
//! in this order, because each depends on the previous one having run:
//!
//! 1. [`Feature::extensions`]: register implementations of the extension
//!    traits. Runs before the core's services are built, since they read the
//!    registry when they are.
//! 2. [`Feature::prepare`]: run against the shared pool before anything
//!    serves (migrations, seeds).
//! 3. [`Feature::install`]: contribute services to the listener. Runs after
//!    the core's services exist, so a feature's own use cases can be built on
//!    the same ports the core uses, through [`Context`].

use crate::surface::Surface;
use scylla_auth::authz::{PermissionService, PolicyControl, VisibilityResolver};
use scylla_extension::Extensions;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// The future type [`Feature::prepare`] returns, so the trait stays object safe.
pub type PrepareFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// What the core hands a feature when it installs: the shared pool, the
/// registry as the core saw it, and the core's own authorization ports, so a
/// feature's use cases enforce the same policies (and see the same reloads)
/// as the core's. Grows by fields, never by parameters.
#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
    pub extensions: Extensions,
    pub permissions: Arc<dyn PermissionService>,
    pub policy_control: Arc<dyn PolicyControl>,
    pub visibility: Arc<dyn VisibilityResolver>,
}

pub trait Feature: Send + 'static {
    /// Register this feature's implementations of the extension traits.
    fn extensions(&self, _registry: &mut Extensions) {}

    /// Run against the shared pool before the server starts.
    fn prepare<'a>(&'a self, _db: &'a PgPool) -> PrepareFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    /// Put this feature's services on the listener.
    fn install(self: Box<Self>, _ctx: &Context, _surface: &mut Surface) {}
}
