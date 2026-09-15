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

pub struct AppTokenOutcome {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Unknown app, inactive app, disabled secret and wrong secret all return the same opaque error.
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

        // Verify every candidate, no early break, so timing does not leak which secret matched.
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
