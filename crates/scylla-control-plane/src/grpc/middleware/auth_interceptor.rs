use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use scylla_core::application::{AppTokenRepository, CallerContext, SessionRepository};
use std::sync::Arc;
use tonic::{Request, Status};
use tonic_async_interceptor::AsyncInterceptor;

/// Authenticated principal attached to each request by the auth interceptor.
/// Either a user (resolved from a session) or a machine App (from an app token).
#[derive(Debug, Clone, Constructor)]
pub struct AuthContext {
    pub caller: CallerContext,
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
pub struct AuthInterceptor<R, AT> {
    session_repo: Arc<R>,
    app_token_repo: Arc<AT>,
}

impl<R, AT> AuthInterceptor<R, AT> {
    pub fn new(session_repo: Arc<R>, app_token_repo: Arc<AT>) -> Self {
        Self {
            session_repo,
            app_token_repo,
        }
    }
}

impl<R, AT> AsyncInterceptor for AuthInterceptor<R, AT>
where
    R: SessionRepository + Send + Sync + 'static,
    AT: AppTokenRepository + Send + Sync + 'static,
{
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Request<()>, Status>> + Send>>;
    fn call(&mut self, mut request: Request<()>) -> Self::Future {
        let session_repo = self.session_repo.clone();
        let app_token_repo = self.app_token_repo.clone();

        Box::pin(async move {
            let token = extract_bearer_token(&request)?;

            // A user session takes precedence; an expired one is swept. Only a
            // genuine "not found" falls through to the App-token path — any other
            // error (DB down, pool exhausted) is a real failure and must surface
            // as INTERNAL, not be masked as an authentication failure.
            match session_repo.find_by_token(&token).await {
                Ok(session) => {
                    if session.is_expired() {
                        let _ = session_repo.delete_by_token(&token).await;
                        return Err(Status::unauthenticated("Token has expired"));
                    }
                    request
                        .extensions_mut()
                        .insert(AuthContext::new(CallerContext::User(
                            session.user_id().clone(),
                        )));
                    return Ok(request);
                }
                Err(e) if e.is_not_found() => {}
                Err(e) => return Err(domain_error_to_status(e)),
            }

            // Otherwise the token may belong to a machine App.
            match app_token_repo.find_by_token(&token).await {
                Ok(app_token) => {
                    if app_token.is_expired() {
                        return Err(Status::unauthenticated("Token has expired"));
                    }
                    request
                        .extensions_mut()
                        .insert(AuthContext::new(CallerContext::App(
                            app_token.app_id().clone(),
                        )));
                    return Ok(request);
                }
                Err(e) if e.is_not_found() => {}
                Err(e) => return Err(domain_error_to_status(e)),
            }

            Err(Status::unauthenticated("Invalid or expired token"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Duration;
    use scylla_core::application::{AppTokenRepository, SessionRepository};
    use scylla_core::domain::entities::{AppCredentialId, AppId, AppToken, Session, UserId};
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

    struct StubAppTokenRepo {
        find_by_token_fn: Box<dyn Fn(&str) -> DomainResult<AppToken> + Send + Sync>,
    }

    #[async_trait]
    impl AppTokenRepository for StubAppTokenRepo {
        async fn create(&self, _t: &AppToken) -> DomainResult<()> {
            unimplemented!()
        }
        async fn find_by_token(&self, token: &str) -> DomainResult<AppToken> {
            (self.find_by_token_fn)(token)
        }
    }

    /// An app-token repo that never matches — for the user-only test paths.
    fn no_app_tokens() -> Arc<StubAppTokenRepo> {
        Arc::new(StubAppTokenRepo {
            find_by_token_fn: Box::new(|t| Err(DomainError::not_found("AppToken", t))),
        })
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
        assert!(extract_auth_context(&req).is_err());
    }

    #[test]
    fn extract_auth_context_present() {
        let mut req = Request::new(());
        let user_id = UserId::generate();
        req.extensions_mut()
            .insert(AuthContext::new(CallerContext::User(user_id.clone())));

        let ctx = extract_auth_context(&req).unwrap();
        assert_eq!(ctx.caller, CallerContext::User(user_id));
    }

    #[tokio::test]
    async fn interceptor_valid_session_token() {
        let session = valid_session();
        let user_id = session.user_id().clone();
        let s = session.clone();

        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(move |_| Ok(s.clone())),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });

        let mut interceptor = AuthInterceptor::new(repo, no_app_tokens());
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer valid-token".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_ok());
        let req = result.unwrap();
        let ctx = req.extensions().get::<AuthContext>().unwrap();
        assert_eq!(ctx.caller, CallerContext::User(user_id));
    }

    #[tokio::test]
    async fn interceptor_app_token_resolves_to_app() {
        // Session lookup misses; the same token resolves to an App principal.
        let app_id = AppId::new("agent-1");
        let token = AppToken::create(
            app_id.clone(),
            AppCredentialId::new("secret-1"),
            "app-token".to_string(),
            Duration::hours(24),
        );
        let t = token.clone();

        let session_repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(|tok| Err(DomainError::not_found("Session", tok))),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });
        let app_repo = Arc::new(StubAppTokenRepo {
            find_by_token_fn: Box::new(move |_| Ok(t.clone())),
        });

        let mut interceptor = AuthInterceptor::new(session_repo, app_repo);
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer app-token".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_ok());
        let req = result.unwrap();
        let ctx = req.extensions().get::<AuthContext>().unwrap();
        assert_eq!(ctx.caller, CallerContext::App(app_id));
    }

    #[tokio::test]
    async fn interceptor_missing_auth_header() {
        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(|_| unreachable!()),
            delete_by_token_fn: Box::new(|_| unreachable!()),
        });

        let mut interceptor = AuthInterceptor::new(repo, no_app_tokens());
        let req = Request::new(());
        let result = interceptor.call(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn interceptor_expired_session() {
        let session = expired_session();
        let s = session.clone();

        let repo = Arc::new(StubSessionRepo {
            find_by_token_fn: Box::new(move |_| Ok(s.clone())),
            delete_by_token_fn: Box::new(|_| Ok(())),
        });

        let mut interceptor = AuthInterceptor::new(repo, no_app_tokens());
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

        let mut interceptor = AuthInterceptor::new(repo, no_app_tokens());
        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer unknown".parse().unwrap());

        let result = interceptor.call(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }
}
