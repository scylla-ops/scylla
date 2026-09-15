use uuid::Uuid;

/// Here and not in the kernel: the random source is a security decision, and `uuid` stays out of the agent.
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
