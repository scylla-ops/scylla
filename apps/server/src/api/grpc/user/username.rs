use protocol::Serialize;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::fmt;
use thiserror::Error;

pub const USERNAME_MIN_LENGTH: usize = 1;
pub const USERNAME_MAX_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScyllaUsername {
    inner: String,
}

impl<'de> Deserialize<'de> for ScyllaUsername {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ScyllaUsername::new(s).map_err(Error::custom)
    }
}

#[derive(Debug, Error)]
pub enum UsernameError {
    #[error("Username too short, min length is {0}")]
    TooShort(usize),
    #[error("Username too long, max length is {0}")]
    TooLong(usize),
}

impl ScyllaUsername {
    pub fn new(value: impl Into<String>) -> Result<Self, UsernameError> {
        let value = value.into();
        let trimmed = value.trim();
        let len = trimmed.chars().count();

        if len < USERNAME_MIN_LENGTH {
            return Err(UsernameError::TooShort(USERNAME_MIN_LENGTH));
        }
        if len > USERNAME_MAX_LENGTH {
            return Err(UsernameError::TooLong(USERNAME_MAX_LENGTH));
        }

        Ok(Self {
            inner: trimmed.to_string(),
        })
    }
}

impl fmt::Display for ScyllaUsername {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
