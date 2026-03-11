use derive_more::Constructor;
use domain::entities::UserId;
use domain::ports::SessionRepository;
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
