use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::AuthUseCases;
use scylla_core::application::ports::{HashService, SessionRepository, UserRepository};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::user::{Password, Username};
use scylla_protocol::services::auth::{
    LoginRequest, LoginResponse, RevokeTokenRequest, RevokeTokenResponse, ValidateTokenRequest,
    ValidateTokenResponse, auth_service_server::AuthService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct AuthHandler<U: UserRepository, S: SessionRepository, H: HashService> {
    use_cases: Arc<AuthUseCases<U, S, H>>,
}

#[async_trait::async_trait]
impl<
    U: UserRepository + Send + Sync + 'static,
    S: SessionRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
> AuthService for AuthHandler<U, S, H>
{
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let (token, user_id) = self
            .use_cases
            .login(username, password)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(LoginResponse {
            token,
            user_id: user_id.to_string(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        if req.token.is_empty() {
            return Ok(Response::new(ValidateTokenResponse {
                is_valid: Some(false),
            }));
        }

        let is_valid = self
            .use_cases
            .validate_token(&req.token)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ValidateTokenResponse {
            is_valid: Some(is_valid),
        }))
    }

    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();

        if req.token.is_empty() {
            return Err(Status::invalid_argument("Token cannot be empty"));
        }

        self.use_cases
            .revoke_token(&req.token)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RevokeTokenResponse {}))
    }
}

impl<U: UserRepository, S: SessionRepository, H: HashService> AuthHandler<U, S, H> {
    pub async fn get_user_id_from_token(&self, token: &str) -> Result<UserId, Status> {
        if token.is_empty() {
            return Err(Status::unauthenticated("Token cannot be empty"));
        }
        self.use_cases
            .get_user_id_from_token(token)
            .await
            .map_err(domain_error_to_status)
    }

    pub async fn touch_session(&self, token: &str) -> Result<(), Status> {
        self.use_cases
            .touch_session(token)
            .await
            .map_err(domain_error_to_status)
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64, Status> {
        self.use_cases
            .cleanup_expired()
            .await
            .map_err(domain_error_to_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use scylla_core::application::AuthUseCases;
    use scylla_core::application::ports::{HashService, SessionRepository, UserRepository};
    use scylla_core::domain::entities::{Session, User};
    use scylla_core::domain::errors::{DomainError, DomainResult};
    use scylla_core::domain::value_objects::user::{Password, PasswordHash, Username};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use scylla_protocol::services::auth::auth_service_server::AuthService;
    use std::sync::Arc;

    // ── Stubs ──��──────────────────────────────────────────────────

    #[derive(Default)]
    struct StubUserRepo {
        find_by_username_fn: Option<Box<dyn Fn(&Username) -> DomainResult<User> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: &UserId) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_username(&self, u: &Username) -> DomainResult<User> {
            (self.find_by_username_fn.as_ref().unwrap())(u)
        }
        async fn update(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            unimplemented!()
        }
        async fn username_exists(&self, _u: &Username) -> DomainResult<bool> {
            unimplemented!()
        }
    }

    struct StubSessionRepo {
        create_fn: Option<Box<dyn Fn(&Session) -> DomainResult<Session> + Send + Sync>>,
        find_by_token_fn: Option<Box<dyn Fn(&str) -> DomainResult<Session> + Send + Sync>>,
        delete_by_token_fn: Option<Box<dyn Fn(&str) -> DomainResult<()> + Send + Sync>>,
    }

    impl Default for StubSessionRepo {
        fn default() -> Self {
            Self {
                create_fn: None,
                find_by_token_fn: None,
                delete_by_token_fn: None,
            }
        }
    }

    #[async_trait]
    impl SessionRepository for StubSessionRepo {
        async fn create(&self, s: &Session) -> DomainResult<Session> {
            (self.create_fn.as_ref().unwrap())(s)
        }
        async fn find_by_token(&self, t: &str) -> DomainResult<Session> {
            (self.find_by_token_fn.as_ref().unwrap())(t)
        }
        async fn update(&self, _s: &Session) -> DomainResult<Session> {
            unimplemented!()
        }
        async fn delete_by_token(&self, t: &str) -> DomainResult<()> {
            (self.delete_by_token_fn.as_ref().unwrap())(t)
        }
        async fn delete_expired(&self) -> DomainResult<u64> {
            unimplemented!()
        }
        async fn list_for_user(&self, _uid: &UserId) -> DomainResult<Vec<Session>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubHash {
        verify_fn:
            Option<Box<dyn Fn(&Password, &PasswordHash) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl HashService for StubHash {
        async fn hash(&self, _p: &Password) -> DomainResult<PasswordHash> {
            unimplemented!()
        }
        async fn verify(&self, p: &Password, h: &PasswordHash) -> DomainResult<bool> {
            (self.verify_fn.as_ref().unwrap())(p, h)
        }
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn test_user() -> User {
        User::create(
            Username::new("admin").unwrap(),
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        )
    }

    fn make_handler(
        user_repo: StubUserRepo,
        session_repo: StubSessionRepo,
        hash: StubHash,
    ) -> AuthHandler<StubUserRepo, StubSessionRepo, StubHash> {
        let uc = Arc::new(AuthUseCases::new(
            Arc::new(user_repo),
            Arc::new(session_repo),
            Arc::new(hash),
        ));
        AuthHandler::new(uc)
    }

    // ── Tests ──────────────────────��──────────────────────────────

    #[tokio::test]
    async fn login_returns_token() {
        let user = test_user();
        let u = user.clone();

        let mut user_repo = StubUserRepo::default();
        user_repo.find_by_username_fn = Some(Box::new(move |_| Ok(u.clone())));

        let mut hash = StubHash::default();
        hash.verify_fn = Some(Box::new(|_, _| Ok(true)));

        let mut session_repo = StubSessionRepo::default();
        session_repo.create_fn = Some(Box::new(|s| Ok(s.clone())));

        let handler = make_handler(user_repo, session_repo, hash);
        let req = Request::new(LoginRequest {
            username: "admin".into(),
            password: "ValidPass123".into(),
        });

        let resp = handler.login(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(!inner.token.is_empty());
        assert!(!inner.user_id.is_empty());
    }

    #[tokio::test]
    async fn login_bad_credentials_returns_unauthenticated() {
        let mut user_repo = StubUserRepo::default();
        user_repo.find_by_username_fn =
            Some(Box::new(|_| Err(DomainError::not_found("User", "x"))));

        let handler = make_handler(user_repo, StubSessionRepo::default(), StubHash::default());
        let req = Request::new(LoginRequest {
            username: "bad".into(),
            password: "ValidPass123".into(),
        });

        let err = handler.login(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn validate_token_returns_valid() {
        let user = test_user();
        let session = Session::create(user.id().clone(), "tok".into(), Duration::hours(24));
        let s = session.clone();

        let mut session_repo = StubSessionRepo::default();
        session_repo.find_by_token_fn = Some(Box::new(move |_| Ok(s.clone())));

        let handler = make_handler(StubUserRepo::default(), session_repo, StubHash::default());
        let req = Request::new(ValidateTokenRequest {
            token: "tok".into(),
        });

        let resp = handler.validate_token(req).await.unwrap();
        assert_eq!(resp.into_inner().is_valid, Some(true));
    }

    #[tokio::test]
    async fn validate_empty_token_returns_false() {
        let handler = make_handler(
            StubUserRepo::default(),
            StubSessionRepo::default(),
            StubHash::default(),
        );
        let req = Request::new(ValidateTokenRequest {
            token: String::new(),
        });

        let resp = handler.validate_token(req).await.unwrap();
        assert_eq!(resp.into_inner().is_valid, Some(false));
    }

    #[tokio::test]
    async fn revoke_token_success() {
        let mut session_repo = StubSessionRepo::default();
        session_repo.delete_by_token_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(StubUserRepo::default(), session_repo, StubHash::default());
        let req = Request::new(RevokeTokenRequest {
            token: "tok".into(),
        });

        assert!(handler.revoke_token(req).await.is_ok());
    }

    #[tokio::test]
    async fn revoke_empty_token_returns_error() {
        let handler = make_handler(
            StubUserRepo::default(),
            StubSessionRepo::default(),
            StubHash::default(),
        );
        let req = Request::new(RevokeTokenRequest {
            token: String::new(),
        });

        let err = handler.revoke_token(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::InvalidArgument);
    }
}
