use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Working directory cannot be empty"));
    }
    if s.starts_with('/') || s.starts_with('\\') {
        return Err(DomainError::validation(
            "Working directory must be relative to the job workspace",
        ));
    }
    // Reject parent traversal so a node cannot escape its job workspace. The
    // agent additionally canonicalizes and prefix-checks at spawn time.
    if s.split(['/', '\\']).any(|c| c == "..") {
        return Err(DomainError::validation(
            "Working directory must not contain `..`",
        ));
    }
    Ok(())
}

/// A node's working directory, relative to the per-job workspace root. Rejects
/// absolute paths and `..` traversal up front.
#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Borrow, Display, Into, Serialize, Deserialize),
)]
pub struct WorkingDir(String);

impl WorkingDir {
    /// Construct from anything string-like.
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_relative_paths() {
        for p in ["crates/api", "build", "a/b/c"] {
            assert!(WorkingDir::new(p).is_ok(), "{p} should be valid");
        }
    }

    #[test]
    fn rejects_absolute_and_traversal() {
        for p in ["", "/etc", "../escape", "a/../b", "/", "\\windows"] {
            assert!(WorkingDir::new(p).is_err(), "{p} should be rejected");
        }
    }
}
