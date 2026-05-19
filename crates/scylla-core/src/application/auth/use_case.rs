use crate::application::{HashService, SessionRepository, UserRepository};
use crate::domain::entities::{Session, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::{Password, Username};
use chrono::Duration;
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

const DEFAULT_SESSION_DURATION_HOURS: i64 = 24;

#[derive(Constructor)]
pub struct AuthUseCases<U: UserRepository, S: SessionRepository, H: HashService> {
    user_repo: Arc<U>,
    session_repo: Arc<S>,
    hash_service: Arc<H>,
}

impl<U: UserRepository, S: SessionRepository, H: HashService> AuthUseCases<U, S, H> {
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn login(
        &self,
        username: Username,
        password: Password,
    ) -> DomainResult<(String, UserId)> {
        let user = self
            .user_repo
            .find_by_username(&username)
            .await
            .map_err(|_| DomainError::unauthorized("Invalid username or password"))?;

        if !user.is_active() {
            return Err(DomainError::unauthorized("User account is inactive"));
        }

        let is_valid = self
            .hash_service
            .verify(&password, user.password_hash())
            .await?;
        if !is_valid {
            return Err(DomainError::unauthorized("Invalid username or password"));
        }

        let token = Uuid::new_v4().to_string();
        let session = Session::create(
            user.id().clone(),
            token.clone(),
            Duration::hours(DEFAULT_SESSION_DURATION_HOURS),
        );
        self.session_repo.create(&session).await?;

        Ok((token, user.id().clone()))
    }

    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> DomainResult<bool> {
        let Ok(session) = self.session_repo.find_by_token(token).await else {
            return Ok(false);
        };

        if session.is_expired() {
            let _ = self.session_repo.delete_by_token(token).await;
            return Ok(false);
        }

        Ok(true)
    }

    #[instrument(skip(self, token))]
    pub async fn revoke_token(&self, token: &str) -> DomainResult<()> {
        self.session_repo.delete_by_token(token).await
    }

    #[instrument(skip(self, token))]
    pub async fn get_user_id_from_token(&self, token: &str) -> DomainResult<UserId> {
        let session = self
            .session_repo
            .find_by_token(token)
            .await
            .map_err(|_| DomainError::unauthorized("Invalid or expired token"))?;

        if session.is_expired() {
            let _ = self.session_repo.delete_by_token(token).await;
            return Err(DomainError::unauthorized("Token has expired"));
        }

        Ok(session.user_id().clone())
    }

    #[instrument(skip(self, token))]
    pub async fn touch_session(&self, token: &str) -> DomainResult<()> {
        let mut session = self.session_repo.find_by_token(token).await?;
        session.touch();
        self.session_repo.update(&session).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> DomainResult<u64> {
        self.session_repo.delete_expired().await
    }
}
