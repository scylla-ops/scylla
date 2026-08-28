use uuid::Uuid;

/// Mint the secret token that lets an invited person join an organization.
///
/// Lives here rather than inside `Invitation::create` for the same reason as
/// [`mint_app_secret`](crate::application::app::mint_app_secret): whoever holds
/// this token gets the access, so choosing a random source is a security
/// decision and not a rule about what an invitation is. Keeping it here also
/// keeps `uuid`, and its RNG, out of the kernel that every agent links.
#[must_use]
pub fn mint_invitation_token() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mints_a_distinct_token_each_time() {
        assert_ne!(mint_invitation_token(), mint_invitation_token());
    }
}
