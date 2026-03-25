use log::error;
use scylla_core::domain::errors::DomainError;
use tonic::Status;

/// Convert DomainError to tonic Status
/// This is a standalone function instead of a From impl due to Rust's orphan rules
pub fn domain_error_to_status(err: DomainError) -> Status {
    match err {
        DomainError::NotFound { entity_type, id } => {
            Status::not_found(format!("{} with id '{}' not found", entity_type, id))
        }
        DomainError::Validation(message) => Status::invalid_argument(message),
        DomainError::BusinessRule(message) => Status::failed_precondition(message),
        DomainError::Unauthorized(message) => Status::unauthenticated(message),
        DomainError::Forbidden(message) => Status::permission_denied(message),
        DomainError::Conflict(message) => Status::already_exists(message),
        DomainError::Infrastructure(message) => {
            error!("Infrastructure error: {}", message);
            Status::internal("Internal server error")
        }
        DomainError::Internal(message) => {
            error!("Internal error: {}", message);
            Status::internal("Internal server error")
        }
    }
}
