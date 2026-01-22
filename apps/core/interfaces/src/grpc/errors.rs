use domain::errors::DomainError;
use log::error;
use tonic::Status;

pub trait ToStatus {
    fn to_status(&self) -> Status;
}

impl ToStatus for DomainError {
    fn to_status(&self) -> Status {
        match self {
            DomainError::NotFound { entity_type, id } => {
                Status::not_found(format!("{} with id '{}' not found", entity_type, id))
            }
            DomainError::Validation(message) => Status::invalid_argument(message.clone()),
            DomainError::BusinessRule(message) => Status::failed_precondition(message.clone()),
            DomainError::Unauthorized(message) => Status::unauthenticated(message.clone()),
            DomainError::Forbidden(message) => Status::permission_denied(message.clone()),
            DomainError::Conflict(message) => Status::already_exists(message.clone()),
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
}
