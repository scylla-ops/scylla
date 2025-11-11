use crate::domain::errors::DomainError;
use protocol::tonic::Status;

impl From<DomainError> for Status {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::NotFound { entity_type, id } => {
                Status::not_found(format!("{} with id '{}' not found", entity_type, id))
            }
            DomainError::Validation(message) => Status::invalid_argument(message),
            DomainError::BusinessRule(message) => Status::failed_precondition(message),
            DomainError::Unauthorized(message) => Status::unauthenticated(message),
            DomainError::Forbidden(message) => Status::permission_denied(message),
            DomainError::Conflict(message) => Status::already_exists(message),
            DomainError::Infrastructure(message) => {
                tracing::error!("Infrastructure error: {}", message);
                Status::internal("Internal server error")
            }
            DomainError::Internal(message) => {
                tracing::error!("Internal error: {}", message);
                Status::internal("Internal server error")
            }
        }
    }
}
