//! `Session` test fixtures.

use bon::bon;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::domain::clock;

use crate::domain::entities::{Session, SessionId, UserId};

pub struct SessionBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl SessionBuilder {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn)] user_id: &UserId,
        id: Option<SessionId>,
        #[builder(into)] token: Option<String>,
        created_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
        last_active_at: Option<DateTime<Utc>>,
        /// Convenience: when `true`, force `created_at`, `expires_at` and
        /// `last_active_at` to be in the past (overrides explicit values).
        #[builder(default = false)]
        expired: bool,
    ) -> Session {
        let (created_at, expires_at, last_active_at) = if expired {
            let now = clock::now();
            (
                Some(now - Duration::hours(2)),
                Some(now - Duration::hours(1)),
                Some(now - Duration::hours(2)),
            )
        } else {
            (created_at, expires_at, last_active_at)
        };
        let now = created_at.unwrap_or_else(clock::now);
        Session::from_persistence(
            id.unwrap_or_else(SessionId::generate),
            token.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id.clone(),
            now,
            expires_at.unwrap_or(now + Duration::hours(1)),
            last_active_at.unwrap_or(now),
        )
    }
}

#[must_use]
pub fn session(user_id: &UserId) -> Session {
    SessionBuilder::new(user_id).build()
}

pub async fn seed_session(pool: &sqlx::PgPool, user_id: &UserId) -> Session {
    use crate::application::SessionRepository;
    use crate::infrastructure::persistence::postgres::PgSessionRepository;
    let session = session(user_id);
    PgSessionRepository::new(pool.clone())
        .create(&session)
        .await
        .expect("seed session failed");
    session
}
