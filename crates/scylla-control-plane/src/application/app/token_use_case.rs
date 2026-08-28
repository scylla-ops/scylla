use crate::application::HashService;
use crate::application::app::credential_repository::AppCredentialRepository;
use crate::application::app::repository::AppRepository;
use crate::application::app::token_repository::AppTokenRepository;
use crate::domain::app::AppSecret;
use crate::domain::app::AppToken;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::AppId;
use chrono::{DateTime, Duration, Utc};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const DEFAULT_APP_TOKEN_DURATION_DAYS: i64 = 30;

/// What a successful token issuance returns to the transport layer.
pub struct AppTokenOutcome {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Exchanges an App's id + the plaintext of any of its *enabled* secrets for a
/// bearer token. Public (takes no `CallerContext`) — the secret is the
/// credential, mirroring user login. An unknown app, an inactive app, a
/// disabled/revoked secret, and a wrong secret all return the same opaque error
/// so callers can't probe which apps or secrets exist.
#[derive(Constructor)]
pub struct AppTokenUseCases<A, T, C, H>
where
    A: AppRepository,
    T: AppTokenRepository,
    C: AppCredentialRepository,
    H: HashService,
{
    app_repo: Arc<A>,
    token_repo: Arc<T>,
    credential_repo: Arc<C>,
    hash_service: Arc<H>,
}

impl<A, T, C, H> AppTokenUseCases<A, T, C, H>
where
    A: AppRepository,
    T: AppTokenRepository,
    C: AppCredentialRepository,
    H: HashService,
{
    #[instrument(skip_all, fields(app_id = %app_id))]
    pub async fn issue(&self, app_id: AppId, secret: AppSecret) -> DomainResult<AppTokenOutcome> {
        let invalid = || DomainError::unauthorized("Invalid app credentials");
        let app = self
            .app_repo
            .find_by_id(&app_id)
            .await
            .map_err(|_| invalid())?;
        if !app.is_active() {
            return Err(invalid());
        }

        // Accept the plaintext against any enabled secret. Verify all candidates
        // (no early break) so timing doesn't leak which secret matched, but keep
        // the matched one so the token is tied to it (revoke/disable that secret
        // → this token dies).
        let credentials = self
            .credential_repo
            .list_enabled_by_app(&app_id)
            .await
            .map_err(|_| invalid())?;
        let mut matched: Option<&_> = None;
        for credential in &credentials {
            if self
                .hash_service
                .verify_secret(&secret, credential.secret_hash())
                .await?
            {
                matched = Some(credential);
            }
        }
        let matched = matched.ok_or_else(invalid)?;

        let token = Uuid::new_v4().to_string();
        let app_token = AppToken::create(
            app_id,
            matched.id().clone(),
            token.clone(),
            Duration::days(DEFAULT_APP_TOKEN_DURATION_DAYS),
        );
        self.token_repo.create(&app_token).await?;

        Ok(AppTokenOutcome {
            token,
            expires_at: app_token.expires_at(),
        })
    }
}
