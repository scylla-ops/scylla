use crate::application::HashService;
use crate::application::app::repository::AppRepository;
use crate::application::app::token_repository::AppTokenRepository;
use crate::domain::entities::{AppId, AppToken};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::app::AppSecret;
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

/// Exchanges an App's id + secret for a bearer token. Public (takes no
/// `CallerContext`) — the secret is the credential, mirroring user login. An
/// unknown app and a wrong secret return the same opaque error so callers can't
/// probe which apps exist.
#[derive(Constructor)]
pub struct AppTokenUseCases<A: AppRepository, T: AppTokenRepository, H: HashService> {
    app_repo: Arc<A>,
    token_repo: Arc<T>,
    hash_service: Arc<H>,
}

impl<A: AppRepository, T: AppTokenRepository, H: HashService> AppTokenUseCases<A, T, H> {
    #[instrument(skip(self, secret), fields(app_id = %app_id))]
    pub async fn issue(&self, app_id: AppId, secret: AppSecret) -> DomainResult<AppTokenOutcome> {
        let invalid = || DomainError::unauthorized("Invalid app credentials");
        let app = self.app_repo.find_by_id(&app_id).await.map_err(|_| invalid())?;
        if !app.is_active() {
            return Err(invalid());
        }
        let valid = self
            .hash_service
            .verify_secret(&secret, app.secret_hash())
            .await?;
        if !valid {
            return Err(invalid());
        }

        let token = Uuid::new_v4().to_string();
        let app_token = AppToken::create(
            app_id,
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
