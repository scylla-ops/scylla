use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_NODE_ID_LENGTH: usize = 128;

fn validate(s: &str) -> Result<(), DomainError> {
    if s.is_empty() {
        return Err(DomainError::validation("Node ID cannot be empty"));
    }
    if s.len() > MAX_NODE_ID_LENGTH {
        return Err(DomainError::validation(format!(
            "Node ID cannot exceed {MAX_NODE_ID_LENGTH} characters"
        )));
    }
    if !s
        .chars()
        .all(|c| (c.is_ascii_alphanumeric() && !c.is_ascii_uppercase()) || c == '-' || c == '_')
    {
        return Err(DomainError::validation(
            "Node ID may only contain lowercase alphanumeric characters, hyphens, and underscores",
        ));
    }
    Ok(())
}

#[nutype(
    sanitize(trim),
    validate(with = validate, error = DomainError),
    derive(
        Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, AsRef, Borrow, Display, Into,
        Serialize, Deserialize,
    ),
)]
pub struct NodeId(String);

impl NodeId {
    /// Backwards-compatible constructor that accepts anything convertible to `String`.
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        Self::try_new(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        <Self as AsRef<str>>::as_ref(self)
    }
}
