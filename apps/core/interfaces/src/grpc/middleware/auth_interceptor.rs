use derive_more::Constructor;
use domain::entities::UserId;
use domain::ports::SessionRepository;
use std::sync::Arc;
use tonic::{Request, Status};

/// Authenticated user context attached to each request by the auth interceptor.
#[derive(Debug, Clone, Constructor)]
pub struct AuthContext {
    pub user_id: UserId,
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

/// Returns a tonic interceptor that validates the bearer token on each request
/// and attaches an [`AuthContext`] to the request extensions on success.
///
/// Tonic interceptors are synchronous, so token validation is performed in a
/// dedicated OS thread with its own single-threaded Tokio runtime to avoid
/// blocking the parent executor.
pub fn auth_interceptor<S: SessionRepository + Send + Sync + 'static>(
    session_repo: Arc<S>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |mut req: Request<()>| {
        let token = extract_bearer_token(&req)?;

        let session_repo = session_repo.clone();
        let user_id = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| Status::internal("Failed to build tokio runtime for auth interceptor"))?;
            rt.block_on(async move {
                let session = session_repo
                    .find_by_token(&token)
                    .await
                    .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

                if session.is_expired() {
                    let _ = session_repo.delete_by_token(&token).await;
                    return Err(Status::unauthenticated("Token has expired"));
                }

                Ok(session.user_id().clone())
            })
        })
        .join()
        .map_err(|_| Status::internal("Auth interceptor thread panicked"))??;

        req.extensions_mut().insert(AuthContext::new(user_id));

        Ok(req)
    }
}

/// Extracts the [`AuthContext`] previously attached by the auth interceptor.
pub fn extract_auth_context<T>(request: &Request<T>) -> Result<AuthContext, Status> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| Status::internal("Auth context not found — interceptor may not be configured"))
        .map(|ctx| ctx.clone())
}

/// Validates a session token directly against the repository.
///
/// Returns the associated user ID if the token is valid and not expired.
/// Expired sessions are deleted as a side effect.
pub async fn validate_token<S: SessionRepository>(
    session_repo: &S,
    token: &str,
) -> Result<UserId, Status> {
    if token.is_empty() {
        return Err(Status::unauthenticated("Token cannot be empty"));
    }

    let session = session_repo
        .find_by_token(token)
        .await
        .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

    if session.is_expired() {
        let _ = session_repo.delete_by_token(token).await;
        return Err(Status::unauthenticated("Token has expired"));
    }

    Ok(session.user_id().clone())
}
