use crate::domain::errors::{DomainError, DomainResult};
use std::fmt::Display;

pub(crate) fn map_sqlx(err: sqlx::Error) -> DomainError {
    match err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            // Constraint names and values must not reach the client.
            tracing::debug!(error = %db_err, "unique-constraint violation");
            DomainError::conflict("a resource with these values already exists")
        }
        sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
            tracing::debug!(error = %db_err, "foreign-key violation");
            DomainError::conflict("references a missing or in-use related resource")
        }
        other => {
            tracing::error!(error = %other, "unexpected database error");
            DomainError::infrastructure("database error")
        }
    }
}

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

pub(crate) trait SqlxResultExt<T> {
    fn not_found_as(
        self,
        entity_type: &'static str,
        id: impl Into<String>,
    ) -> crate::domain::errors::DomainResult<T>;

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

pub(crate) trait DbFieldExt<T> {
    fn db_field(self, field: &'static str) -> DomainResult<T>;
}

impl<T, E: Display> DbFieldExt<T> for Result<T, E> {
    fn db_field(self, field: &'static str) -> DomainResult<T> {
        self.map_err(|e| DomainError::infrastructure(format!("invalid {field} in DB: {e}")))
    }
}
