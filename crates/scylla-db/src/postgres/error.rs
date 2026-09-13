use crate::domain::errors::{DomainError, DomainResult};
use std::fmt::Display;

/// Map a `sqlx::Error` to a `DomainError`, recognising unique / FK violations
/// as `Conflict` so callers don't have to special-case them.
pub(crate) fn map_sqlx(err: sqlx::Error) -> DomainError {
    match err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            // The raw driver text (constraint name, column values) must not reach
            // the client; log it for operators and return a generic conflict.
            tracing::debug!(error = %db_err, "unique-constraint violation");
            DomainError::conflict("a resource with these values already exists")
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            tracing::debug!(error = %db_err, "foreign-key violation");
            DomainError::conflict("references a missing or in-use related resource")
        }
        other => {
            // Never surface raw SQL/driver detail to callers.
            tracing::error!(error = %other, "unexpected database error");
            DomainError::infrastructure("database error")
        }
    }
}

/// Translate `sqlx::Error::RowNotFound` into a domain `NotFound`. Other errors
/// go through `map_sqlx`.
pub(crate) fn map_sqlx_not_found(
    err: sqlx::Error,
    entity_type: &str,
    id: impl Into<String>,
) -> DomainError {
    match err {
        sqlx::Error::RowNotFound => DomainError::not_found(entity_type, id),
        other => map_sqlx(other),
    }
}

/// Extension methods on `Result<T, sqlx::Error>` for terse error mapping at
/// query call sites.
pub(crate) trait SqlxResultExt<T> {
    /// Translate `RowNotFound` to `DomainError::NotFound`, anything else via
    /// `map_sqlx`. Equivalent to `.map_err(|e| map_sqlx_not_found(e, …))`.
    fn not_found_as(
        self,
        entity_type: &'static str,
        id: impl Into<String>,
    ) -> crate::domain::errors::DomainResult<T>;

    /// Translate any `sqlx::Error` via `map_sqlx`. Equivalent to
    /// `.map_err(map_sqlx)`.
    fn to_domain(self) -> crate::domain::errors::DomainResult<T>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn not_found_as(
        self,
        entity_type: &'static str,
        id: impl Into<String>,
    ) -> crate::domain::errors::DomainResult<T> {
        self.map_err(|e| map_sqlx_not_found(e, entity_type, id))
    }

    fn to_domain(self) -> crate::domain::errors::DomainResult<T> {
        self.map_err(map_sqlx)
    }
}

/// Extension on any `Result<T, E: Display>` used inside repository row mappers.
///
/// Repositories convert raw column values into validated value objects (`Username::new`,
/// `OrganizationName::new`, …). When a value persisted in the DB violates a current
/// invariant, this is an *infrastructure* error (schema drift, manual edit, code/data
/// version mismatch) — never a domain validation failure for the caller. This helper
/// centralises that translation:
///
/// ```ignore
/// // before
/// OrganizationName::new(name)
///     .map_err(|e| DomainError::infrastructure(format!("invalid org name in DB: {e}")))?
/// // after
/// OrganizationName::new(name).db_field("org name")?
/// ```
pub(crate) trait DbFieldExt<T> {
    fn db_field(self, field: &'static str) -> DomainResult<T>;
}

impl<T, E: Display> DbFieldExt<T> for Result<T, E> {
    fn db_field(self, field: &'static str) -> DomainResult<T> {
        self.map_err(|e| DomainError::infrastructure(format!("invalid {field} in DB: {e}")))
    }
}
