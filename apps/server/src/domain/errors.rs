/// Domain-level errors that represent business rule violations and domain concerns.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// Entity not found in the system
    #[error("Entity not found: {entity_type} with id '{id}'")]
    NotFound { entity_type: String, id: String },

    /// Validation error for domain objects
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Business rule violation
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// Unauthorized access attempt
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden action - user doesn't have permission
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Conflict error - entity already exists or state conflict
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
    /// Create a NotFound error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a Validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create a BusinessRule error
    pub fn business_rule(message: impl Into<String>) -> Self {
        Self::BusinessRule(message.into())
    }

    /// Create an Unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    /// Create a Forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    /// Create a Conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    /// Create an Infrastructure error
    pub fn infrastructure(message: impl Into<String>) -> Self {
        Self::Infrastructure(message.into())
    }

    /// Create an Internal error
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
