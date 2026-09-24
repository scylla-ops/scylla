use crate::application::{AppTokenRepository, SessionRepository};
use crate::domain::caller::CallerContext;
use crate::grpc::mappers::domain_error_to_status;
use derive_more::Constructor;
use std::sync::Arc;
use tonic::{Request, Status};
use tonic_async_interceptor::AsyncInterceptor;

#[derive(Debug, Clone, Constructor)]
pub struct AuthContext {
    pub caller: CallerContext,
}

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
pub struct AuthInterceptor {
    session_repo: Arc<dyn SessionRepository>,
    app_token_repo: Arc<dyn AppTokenRepository>,
}

impl AuthInterceptor {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        app_token_repo: Arc<dyn AppTokenRepository>,
    ) -> Self {
        Self {
            session_repo,
            app_token_repo,
        }
    }
}

impl AsyncInterceptor for AuthInterceptor {
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Request<()>, Status>> + Send>>;
    fn call(&mut self, mut request: Request<()>) -> Self::Future {
        let session_repo = self.session_repo.clone();
        let app_token_repo = self.app_token_repo.clone();

        Box::pin(async move {
            let token = extract_bearer_token(&request)?;

            // Only a genuine not-found falls through to the App token; a DB failure must surface as INTERNAL.
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
    use crate::application::{AppTokenRepository, SessionRepository};
    use async_trait::async_trait;
    use chrono::Duration;
    use scylla_domain::domain::app::AppToken;
    use scylla_domain::domain::errors::{DomainError, DomainResult};
    use scylla_domain::domain::ids::{AppCredentialId, AppId, UserId};
    use scylla_domain::domain::session::Session;
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
