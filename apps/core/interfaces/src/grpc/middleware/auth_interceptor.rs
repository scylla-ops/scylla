use derive_more::Constructor;
use domain::entities::UserId;
use domain::ports::SessionRepository;
use std::sync::Arc;
use tonic::{Request, Status};

/// User context extracted from authentication
#[derive(Debug, Clone, Constructor)]
pub struct AuthContext {
    pub user_id: UserId,
}

/// Extract bearer token from request metadata
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

/// Tonic interceptor that validates session tokens and attaches auth context to requests.
/// This runs before each handler method and validates the bearer token against the session repository.
///
/// # Type Parameters
/// * `S` - A type implementing `SessionRepository`
///
/// # Arguments
/// * `session_repo` - Arc-wrapped session repository for looking up sessions
///
/// # Returns
/// A closure that can be used as a tonic interceptor
pub fn auth_interceptor<S: SessionRepository + 'static>(
    session_repo: Arc<S>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |mut req: Request<()>| {
        // Extract bearer token from request
        let token = extract_bearer_token(&req)?;

        // Validate token and extract user_id synchronously by blocking on the async operation
        // Note: This is necessary because tonic interceptors are sync functions
        let session_repo = session_repo.clone();
        let user_id = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Find session by token
                let session = session_repo
                    .find_by_token(&token)
                    .await
                    .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

                // Check if session is expired
                if session.is_expired() {
                    // Attempt to clean up expired session (don't fail if cleanup fails)
                    let _ = session_repo.delete_by_token(&token).await;
                    return Err(Status::unauthenticated("Token has expired"));
                }

                Ok(session.user_id().clone())
            })
        })?;

        // Store the validated user ID in request extensions for handlers to use
        req.extensions_mut().insert(AuthContext::new(user_id));

        Ok(req)
    }
}

/// Extract authenticated user context from request.
/// Does not check permissions, just extracts the authenticated user ID.
///
/// # Arguments
/// * `request` - The gRPC request to extract auth context from
///
/// # Returns
/// The `AuthContext` if present, or an error status if the interceptor wasn't configured
pub fn extract_auth_context<T>(request: &Request<T>) -> Result<AuthContext, Status> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| {
            Status::internal("Auth context not found - interceptor may not be configured")
        })
        .map(|ctx| ctx.clone())
}

/// Validate a token directly without going through the interceptor.
/// Useful for endpoints that need to validate tokens programmatically.
///
/// # Arguments
/// * `session_repo` - The session repository to use for validation
/// * `token` - The token to validate
///
/// # Returns
/// The user ID if the token is valid, or an error status
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
        // Attempt to clean up expired session
        let _ = session_repo.delete_by_token(token).await;
        return Err(Status::unauthenticated("Token has expired"));
    }

    Ok(session.user_id().clone())
}
