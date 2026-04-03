use crate::application::ports::{HashService, SessionRepository, UserRepository};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{HashService, SessionRepository, UserRepository};
    use crate::domain::entities::User;
    use crate::domain::value_objects::user::{Password, PasswordHash, Username};
    use crate::domain::value_objects::{PaginatedResult, PaginationParams};
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Manual mock: UserRepository ───────────────────────────────

    #[derive(Default)]
    struct StubUserRepo {
        find_by_username_fn:
            Option<Box<dyn Fn(&Username) -> DomainResult<User> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _user: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: &UserId) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
            (self.find_by_username_fn.as_ref().unwrap())(username)
        }
        async fn update(&self, _user: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _pagination: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            unimplemented!()
        }
        async fn username_exists(&self, _username: &Username) -> DomainResult<bool> {
            unimplemented!()
        }
    }

    // ── Manual mock: SessionRepository ────────────────────────────

    struct StubSessionRepo {
        create_fn: Option<Box<dyn Fn(&Session) -> DomainResult<Session> + Send + Sync>>,
        find_by_token_fn: Option<Box<dyn Fn(&str) -> DomainResult<Session> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Session) -> DomainResult<Session> + Send + Sync>>,
        delete_by_token_fn: Option<Box<dyn Fn(&str) -> DomainResult<()> + Send + Sync>>,
        delete_expired_fn: Option<Box<dyn Fn() -> DomainResult<u64> + Send + Sync>>,
    }

    impl Default for StubSessionRepo {
        fn default() -> Self {
            Self {
                create_fn: None,
                find_by_token_fn: None,
                update_fn: None,
                delete_by_token_fn: None,
                delete_expired_fn: None,
            }
        }
    }

    #[async_trait]
    impl SessionRepository for StubSessionRepo {
        async fn create(&self, session: &Session) -> DomainResult<Session> {
            (self.create_fn.as_ref().unwrap())(session)
        }
        async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
            (self.find_by_token_fn.as_ref().unwrap())(token)
        }
        async fn update(&self, session: &Session) -> DomainResult<Session> {
            (self.update_fn.as_ref().unwrap())(session)
        }
        async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
            (self.delete_by_token_fn.as_ref().unwrap())(token)
        }
        async fn delete_expired(&self) -> DomainResult<u64> {
            (self.delete_expired_fn.as_ref().unwrap())()
        }
        async fn list_for_user(&self, _user_id: &UserId) -> DomainResult<Vec<Session>> {
            unimplemented!()
        }
    }

    // ── Manual mock: HashService ──────────────────────────────────

    #[derive(Default)]
    struct StubHash {
        verify_fn: Option<Box<dyn Fn(&Password, &PasswordHash) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl HashService for StubHash {
        async fn hash(&self, _password: &Password) -> DomainResult<PasswordHash> {
            unimplemented!()
        }
        async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool> {
            (self.verify_fn.as_ref().unwrap())(password, hash)
        }
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn test_user() -> User {
        let username = Username::new("testuser").unwrap();
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap();
        User::create(username, hash)
    }

    fn test_session(user_id: UserId) -> Session {
        Session::create(user_id, "test-token".to_string(), Duration::hours(24))
    }

    fn expired_session(user_id: UserId) -> Session {
        Session::create(user_id, "expired-token".to_string(), Duration::hours(-1))
    }

    fn make_uc(
        user_repo: StubUserRepo,
        session_repo: StubSessionRepo,
        hash_service: StubHash,
    ) -> AuthUseCases<StubUserRepo, StubSessionRepo, StubHash> {
        AuthUseCases::new(
            Arc::new(user_repo),
            Arc::new(session_repo),
            Arc::new(hash_service),
        )
    }

    // ── Tests ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn login_success() {
        let user = test_user();
        let user_id = user.id().clone();

        let mut user_repo = StubUserRepo::default();
        let u = user.clone();
        user_repo.find_by_username_fn = Some(Box::new(move |_| Ok(u.clone())));

        let mut hash = StubHash::default();
        hash.verify_fn = Some(Box::new(|_, _| Ok(true)));

        let mut session_repo = StubSessionRepo::default();
        session_repo.create_fn = Some(Box::new(|s| Ok(s.clone())));

        let uc = make_uc(user_repo, session_repo, hash);
        let username = Username::new("testuser").unwrap();
        let password = Password::new("ValidPass123").unwrap();

        let (token, returned_id) = uc.login(username, password).await.unwrap();
        assert!(!token.is_empty());
        assert_eq!(returned_id, user_id);
    }

    #[tokio::test]
    async fn login_invalid_username() {
        let mut user_repo = StubUserRepo::default();
        user_repo.find_by_username_fn =
            Some(Box::new(|_| Err(DomainError::not_found("User", "unknown"))));

        let uc = make_uc(user_repo, StubSessionRepo::default(), StubHash::default());
        let username = Username::new("unknown").unwrap();
        let password = Password::new("ValidPass123").unwrap();

        let result = uc.login(username, password).await;
        assert!(matches!(result.unwrap_err(), DomainError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn login_inactive_user() {
        let mut user = test_user();
        user.deactivate().unwrap();

        let mut user_repo = StubUserRepo::default();
        let u = user.clone();
        user_repo.find_by_username_fn = Some(Box::new(move |_| Ok(u.clone())));

        let uc = make_uc(user_repo, StubSessionRepo::default(), StubHash::default());
        let username = Username::new("testuser").unwrap();
        let password = Password::new("ValidPass123").unwrap();

        let result = uc.login(username, password).await;
        assert!(matches!(result.unwrap_err(), DomainError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn login_invalid_password() {
        let user = test_user();
        let mut user_repo = StubUserRepo::default();
        let u = user.clone();
        user_repo.find_by_username_fn = Some(Box::new(move |_| Ok(u.clone())));

        let mut hash = StubHash::default();
        hash.verify_fn = Some(Box::new(|_, _| Ok(false)));

        let uc = make_uc(user_repo, StubSessionRepo::default(), hash);
        let username = Username::new("testuser").unwrap();
        let password = Password::new("WrongPass123").unwrap();

        let result = uc.login(username, password).await;
        assert!(matches!(result.unwrap_err(), DomainError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn validate_token_valid() {
        let user = test_user();
        let session = test_session(user.id().clone());

        let mut session_repo = StubSessionRepo::default();
        let s = session.clone();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert!(uc.validate_token("test-token").await.unwrap());
    }

    #[tokio::test]
    async fn validate_token_expired() {
        let user = test_user();
        let session = expired_session(user.id().clone());

        let mut session_repo = StubSessionRepo::default();
        let s = session.clone();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));
        session_repo.delete_by_token_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert!(!uc.validate_token("expired-token").await.unwrap());
    }

    #[tokio::test]
    async fn validate_token_not_found() {
        let mut session_repo = StubSessionRepo::default();
        session_repo.find_by_token_fn =
            Some(Box::new(|_| Err(DomainError::not_found("Session", "x"))));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert!(!uc.validate_token("unknown").await.unwrap());
    }

    #[tokio::test]
    async fn revoke_token_success() {
        let mut session_repo = StubSessionRepo::default();
        session_repo.delete_by_token_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert!(uc.revoke_token("some-token").await.is_ok());
    }

    #[tokio::test]
    async fn get_user_id_from_valid_token() {
        let user = test_user();
        let user_id = user.id().clone();
        let session = test_session(user.id().clone());

        let mut session_repo = StubSessionRepo::default();
        let s = session.clone();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert_eq!(uc.get_user_id_from_token("test-token").await.unwrap(), user_id);
    }

    #[tokio::test]
    async fn get_user_id_from_expired_token() {
        let user = test_user();
        let session = expired_session(user.id().clone());

        let mut session_repo = StubSessionRepo::default();
        let s = session.clone();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));
        session_repo.delete_by_token_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        let result = uc.get_user_id_from_token("expired-token").await;
        assert!(matches!(result.unwrap_err(), DomainError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn touch_session_success() {
        let user = test_user();
        let session = test_session(user.id().clone());

        let mut session_repo = StubSessionRepo::default();
        let s = session.clone();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));
        session_repo.update_fn = Some(Box::new(|s| Ok(s.clone())));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert!(uc.touch_session("test-token").await.is_ok());
    }

    #[tokio::test]
    async fn cleanup_expired_sessions() {
        let mut session_repo = StubSessionRepo::default();
        session_repo.delete_expired_fn = Some(Box::new(|| Ok(5)));

        let uc = make_uc(StubUserRepo::default(), session_repo, StubHash::default());
        assert_eq!(uc.cleanup_expired().await.unwrap(), 5);
    }
}
