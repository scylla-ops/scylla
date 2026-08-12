use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_NAME_LENGTH: usize = 255;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Role name cannot be empty"));
    }
    if s.len() > MAX_NAME_LENGTH {
        return Err(DomainError::validation(format!(
            "Role name cannot exceed {MAX_NAME_LENGTH} characters"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct RoleName(String);

impl RoleName {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}
