use std::fmt;
use thiserror::Error;

pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScyllaPassword(String);

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password too short, min length is {0}")]
    TooShort(usize),
    #[error("Password too long, max length is {0}")]
    TooLong(usize),
    #[error("Password cannot be empty")]
    Empty,
    #[error("Password cannot be whitespace-only")]
    WhitespaceOnly,
}

impl ScyllaPassword {
    pub fn new(value: impl Into<String>) -> Result<Self, PasswordError> {
        let value = value.into();
        let len = value.chars().count();

        if len == 0 {
            return Err(PasswordError::Empty);
        }
        if value.trim().is_empty() {
            return Err(PasswordError::WhitespaceOnly);
        }
        if len < PASSWORD_MIN_LENGTH {
            return Err(PasswordError::TooShort(PASSWORD_MIN_LENGTH));
        }
        if len > PASSWORD_MAX_LENGTH {
            return Err(PasswordError::TooLong(PASSWORD_MAX_LENGTH));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScyllaPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "*".repeat(self.0.len()))
    }
}
