/// Domain-level errors that represent business rule violations and domain concerns.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Entity not found: {entity_type} with id '{id}'")]
    NotFound { entity_type: String, id: String },

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden - user lacks permission for the requested action
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Conflict - entity already exists or state conflict
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Infrastructure error (database, external services, etc.)
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    /// Internal error - unexpected condition
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience type alias for Results in the domain layer
pub type DomainResult<T> = Result<T, DomainError>;

impl DomainError {
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn business_rule(message: impl Into<String>) -> Self {
        Self::BusinessRule(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn infrastructure(message: impl Into<String>) -> Self {
        Self::Infrastructure(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

// Conversion from anyhow::Error to DomainError for infrastructure layer
impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        DomainError::Infrastructure(err.to_string())
    }
}
