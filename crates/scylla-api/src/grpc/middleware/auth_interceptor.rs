use derive_more::Constructor;
use scylla_core::application::SessionRepository;
use scylla_core::domain::entities::UserId;
use std::sync::Arc;
use tonic::{Request, Status};
use tonic_async_interceptor::AsyncInterceptor;

/// Authenticated user context attached to each request by the auth interceptor.
#[derive(Debug, Clone, Constructor)]
pub struct AuthContext {
    pub user_id: UserId,
}

/// Extracts the [`AuthContext`] previously attached by the auth interceptor.
pub fn extract_auth_context<T>(request: &Request<T>) -> Result<AuthContext, Status> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| {
            Status::internal("Auth context not found — interceptor may not be configured")
        })
        .cloned()
}

fn extract_bearer_token<T>(request: &Request<T>) -> Result<String, Status> {
    let metadata = request.metadata();

    if let Some(auth_header) = metadata.get("authorization") {
        let auth_str = auth_header
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            return Ok(token.to_string());
        }
    }

    Err(Status::unauthenticated(
        "Missing or invalid authorization token",
    ))
}

#[derive(Clone)]
pub struct AuthInterceptor<R> {
    session_repo: Arc<R>,
}

impl<R> AuthInterceptor<R> {
    pub fn new(session_repo: Arc<R>) -> Self {
        Self { session_repo }
    }
}

impl<R> AsyncInterceptor for AuthInterceptor<R>
where
    R: SessionRepository + Send + Sync + 'static,
{
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Request<()>, Status>> + Send>>;
    fn call(&mut self, mut request: Request<()>) -> Self::Future {
        let session_repo = self.session_repo.clone();

        Box::pin(async move {
            let token = extract_bearer_token(&request)?;

            let session = session_repo
                .find_by_token(&token)
                .await
                .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

            if session.is_expired() {
                let _ = session_repo.delete_by_token(&token).await;
                return Err(Status::unauthenticated("Token has expired"));
            }

            request
                .extensions_mut()
                .insert(AuthContext::new(session.user_id().clone()));

            Ok(request)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use scylla_core::application::SessionRepository;
    use scylla_core::domain::entities::{Session, UserId};
    use scylla_core::domain::errors::{DomainError, DomainResult};
    use std::sync::Arc;
    use tonic_async_interceptor::AsyncInterceptor;

    struct StubSessionRepo {
        find_by_token_fn: Box<dyn Fn(&str) -> DomainResult<Session> + Send + Sync>,
        delete_by_token_fn: Box<dyn Fn(&str) -> DomainResult<()> + Send + Sync>,
    }

    #[async_trait]
    impl SessionRepository for StubSessionRepo {
        async fn create(&self, _s: &Session) -> DomainResult<Session> {
            unimplemented!()
        }
        async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
            (self.find_by_token_fn)(token)
        }
        async fn update(&self, _s: &Session) -> DomainResult<Session> {
            unimplemented!()
        }
        async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
            (self.delete_by_token_fn)(token)
        }
        async fn delete_expired(&self) -> DomainResult<u64> {
            unimplemented!()
        }
        async fn list_for_user(&self, _uid: &UserId) -> DomainResult<Vec<Session>> {
            unimplemented!()
        }
    }

    fn valid_session() -> Session {
        Session::create(
            UserId::generate(),
            "valid-token".to_string(),
            Duration::hours(24),
        )
    }

    fn expired_session() -> Session {
        Session::create(
            UserId::generate(),
            "expired-token".to_string(),
            Duration::hours(-1),
        )
    }

    #[test]
    fn extract_auth_context_missing() {
        let req = Request::new(());
        let result = extract_auth_context(&req);
        assert!(result.is_err());
    }

    #[test]
    fn extract_auth_context_present() {
        let mut req = Request::new(());
        let user_id = UserId::generate();
        req.extensions_mut()
            .insert(AuthContext::new(user_id.clone()));

        let ctx = extract_auth_context(&req).unwrap();
        assert_eq!(ctx.user_id, user_id);
    }

    #[tokio::test]
    async fn interceptor_valid_token() {
        let session = valid_session();
        let user_id = session.user_id().clone();
        let s = session.clone();

        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(move |_| Ok(s.clone())),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });

        let mut interceptor = AuthInterceptor::new(repo);
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer valid-token".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_ok());
        let req = result.unwrap();
        let ctx = req.extensions().get::<AuthContext>().unwrap();
        assert_eq!(ctx.user_id, user_id);
    }

    #[tokio::test]
    async fn interceptor_missing_auth_header() {
        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(|_| unreachable!()),
            delete_by_token_fn: Box::new(|_| unreachable!()),
        });

        let mut interceptor = AuthInterceptor::new(repo);
        let req = Request::new(());
        let result = interceptor.call(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn interceptor_expired_token() {
        let session = expired_session();
        let s = session.clone();

        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(move |_| Ok(s.clone())),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });

        let mut interceptor = AuthInterceptor::new(repo);
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer expired-token".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn interceptor_unknown_token() {
        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(|_| Err(DomainError::not_found("Session", "x"))),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });

        let mut interceptor = AuthInterceptor::new(repo);
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer unknown".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }
}
