use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_DESCRIPTION_LENGTH: usize = 1024;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.len() > MAX_DESCRIPTION_LENGTH {
        return Err(DomainError::validation(format!(
            "Description cannot exceed {MAX_DESCRIPTION_LENGTH} characters"
        )));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct ProjectDescription(String);

impl ProjectDescription {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}
