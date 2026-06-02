use crate::domain::errors::{DomainError, DomainResult};
use nutype::nutype;

const MAX_EMAIL_LENGTH: usize = 320; // RFC 5321 upper bound (64 local + @ + 255 domain)

/// Deliberately light validation: a single `@` with a non-empty local part and
/// a domain containing a dot. Full RFC 5322 validation is famously brittle; we
/// rely on email *delivery* (verification mail) as the real check.
fn validate(s: &str) -> Result<(), DomainError> {
    if s.len() > MAX_EMAIL_LENGTH {
        return Err(DomainError::validation(format!(
            "Email cannot exceed {MAX_EMAIL_LENGTH} characters"
        )));
    }
    let Some((local, domain)) = s.split_once('@') else {
        return Err(DomainError::validation("Email must contain '@'"));
    };
    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(DomainError::validation("Email is not a valid address"));
    }
    Ok(())
}

#[nutype(
    sanitize(trim, lowercase),
    validate(with = validate, error = DomainError),
    derive(Debug, Clone, PartialEq, Eq, Hash, AsRef, Borrow, Display, Into),
)]
pub struct Email(String);

impl Email {
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
    fn accepts_valid_and_normalises_case() {
        let e = Email::new("  Alice@Example.COM ").unwrap();
        assert_eq!(e.as_str(), "alice@example.com");
    }

    #[test]
    fn rejects_malformed() {
        assert!(Email::new("nope").is_err());
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("a@b").is_err());
        assert!(Email::new("a@.com").is_err());
        assert!(Email::new("a@b.").is_err());
    }
}
