use crate::api::grpc::auth::service::AuthService;
use crate::api::grpc::rbac::enforcer;
use crate::api::grpc::user::repos::surreal::UserRepositorySurreal;
use protocol::tonic::{Request, Status};

/// extracts user ID from Bearer token in authorization header
pub async fn extract_user_from_token<T>(req: &Request<T>) -> Result<String, Status> {
    let meta = req
        .metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

    let header_value = meta
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

    let token = header_value
        .strip_prefix("Bearer ")
        .ok_or_else(|| Status::unauthenticated("Authorization must be Bearer token"))?;

    // verify the token and extract user ID
    let authed_user = AuthService::<UserRepositorySurreal>::_verify_paseto(token)
        .await
        .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;

    Ok(authed_user._id.to_string())
}

/// checks if a user has permission to access a resource
/// this is a helper function that can be called from controllers
pub async fn check_permission(
    user_id: &str,
    domain_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), Status> {
    let has_permission = enforcer::enforce(user_id, domain_id, resource, action)
        .await
        .map_err(|e| Status::internal(format!("Permission check failed: {}", e)))?;

    if !has_permission {
        return Err(Status::permission_denied(format!(
            "User does not have permission to {} {} in domain {}",
            action, resource, domain_id
        )));
    }

    Ok(())
}

/// gRPC interceptor function for authentication and authorization
/// this can be used with tonic's `with_interceptor` method
///
/// note: for async interceptors, you may need to use a different approach
/// this function is provided as a reference but may need adjustment based on your tonic version
#[allow(dead_code)]
pub fn auth_interceptor<T>(req: Request<T>) -> Result<Request<T>, Status> {
    // for now, we'll just pass through all requests
    // actual authentication should be done in each controller method
    // using extract_user_from_token
    Ok(req)
}

/// extract user ID from request extensions (set by auth_interceptor)
#[allow(dead_code)]
pub fn get_user_from_request<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("User not authenticated"))
}

