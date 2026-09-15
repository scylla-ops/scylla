use scylla_domain::domain::app::AppSecret;
use uuid::Uuid;

/// Here and not in the kernel: the random source is a security decision, and `uuid` stays out of the agent.
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
