use scylla_core::domain::app::AppSecret;
use uuid::Uuid;

/// Mint a fresh app secret: 256 bits of randomness, hex-encoded.
///
/// Lives here rather than on `AppSecret` in the kernel because choosing a random
/// source is a security decision, not a rule about what an app secret is. The
/// kernel owns the shape (length bounds, redacted `Debug`); the control plane
/// owns where the bytes come from. It also keeps `uuid`, and its RNG, out of a
/// crate every agent links.
///
/// Two v4 UUIDs concatenated give 256 bits from the OS RNG, and 64 hex chars
/// always satisfy the length bounds `AppSecret` enforces.
#[must_use]
pub fn mint_app_secret() -> AppSecret {
    let raw = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    AppSecret::new(raw).expect("a 64 hex-char secret is always within bounds")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mints_a_valid_and_unique_secret() {
        let a = mint_app_secret();
        let b = mint_app_secret();
        assert_eq!(a.as_str().len(), 64);
        assert_ne!(a.as_str(), b.as_str());
    }
}
