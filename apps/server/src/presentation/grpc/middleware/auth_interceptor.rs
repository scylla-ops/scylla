/// Authentication interceptor for gRPC services
use crate::application::ports::RbacEnforcer;
use crate::domain::value_objects::UserId;
use crate::shared::di::AppContainer;
use protocol::tonic::{Request, Status};
use std::sync::Arc;

/// User context extracted from authentication
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: UserId,
}

impl AuthContext {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
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

/// Tonic interceptor that validates tokens and attaches auth context to requests
/// This runs before each handler method and validates the bearer token
pub fn auth_interceptor(
    container: Arc<AppContainer>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |mut req: Request<()>| {
        // Extract and validate token
        let token = extract_bearer_token(&req)?;

        // Validate token and extract user_id synchronously by blocking on the async operation
        // Note: This is necessary because tonic interceptors are sync functions
        let auth_service = container.auth_service();
        let user_id = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // First validate the token
                auth_service.validate_token(&token).await.map_err(|e| {
                    Status::unauthenticated(format!("Token validation failed: {}", e))
                })?;

                // Then extract the user_id
                auth_service.extract_user_id(&token).await.map_err(|e| {
                    Status::unauthenticated(format!("Failed to extract user_id: {}", e))
                })
            })
        })?;

        // Store the validated user ID in request extensions for handlers to use
        req.extensions_mut().insert(AuthContext::new(user_id));

        Ok(req)
    }
}

/// Extract authenticated user context from request
/// Does not check permissions, just extracts the authenticated user ID
pub fn extract_auth_context<T>(request: &Request<T>) -> Result<AuthContext, Status> {
    request
        .extensions()
        .get::<AuthContext>()
        .ok_or_else(|| {
            Status::internal("Auth context not found - interceptor may not be configured")
        })
        .map(|ctx| ctx.clone())
}

/// Check RBAC permissions for an authenticated request
/// Extracts the pre-validated AuthContext from request extensions and checks permissions
pub async fn check_permissions<T>(
    request: &Request<T>,
    rbac_enforcer: Arc<dyn RbacEnforcer>,
    domain: &str,
    resource: &str,
    action: &str,
) -> Result<AuthContext, Status> {
    // Get the auth context that was set by the interceptor
    let auth_ctx = extract_auth_context(request)?;

    // Check permissions
    let allowed = rbac_enforcer
        .enforce(&auth_ctx.user_id, domain, resource, action)
        .await
        .map_err(|e| Status::internal(format!("Permission check failed: {}", e)))?;

    if !allowed {
        return Err(Status::permission_denied(format!(
            "User does not have permission to {} {} in domain {}",
            action, resource, domain
        )));
    }

    Ok(auth_ctx)
}
