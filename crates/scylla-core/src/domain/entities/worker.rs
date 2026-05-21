use crate::domain::clock;
use crate::domain::entities::ids::AppId;
use chrono::{DateTime, Utc};

/// The 1:1 specialization of an [`App`] that runs jobs. Its identity and
/// credential live on the `App`; this aggregate only marks "this app is a
/// worker" and carries worker-only attributes. `last_seen` is the durable
/// last-activity timestamp (survives a control-plane restart); live
/// online/offline presence is read from the in-memory worker registry, not here.
///
/// [`App`]: crate::domain::entities::App
#[derive(Debug, Clone)]
pub struct Worker {
    app_id: AppId,
    last_seen: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl Worker {
    #[must_use]
    pub fn from_persistence(
        app_id: AppId,
        last_seen: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            app_id,
            last_seen,
            created_at,
        }
    }

    /// A freshly registered worker has never been seen connected yet.
    #[must_use]
    pub fn create(app_id: AppId) -> Self {
        Self {
            app_id,
            last_seen: None,
            created_at: clock::now(),
        }
    }

    #[must_use]
    pub fn app_id(&self) -> &AppId {
        &self.app_id
    }

    #[must_use]
    pub fn last_seen(&self) -> Option<DateTime<Utc>> {
        self.last_seen
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
