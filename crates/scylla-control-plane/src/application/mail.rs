use crate::domain::errors::DomainResult;
use crate::domain::user::Email;
use async_trait::async_trait;

/// Outbound transactional email. Kept dependency-free (no `lettre` here) so it
/// compiles in every edition; the SMTP implementation lives behind the `mail`
/// feature in the infrastructure layer.
#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, to: &Email, subject: &str, body_html: &str) -> DomainResult<()>;
}

/// No-op mailer: logs and succeeds. Used in tests and as a safe fallback when no
/// SMTP transport is configured.
pub struct NoopMailer;

#[async_trait]
impl Mailer for NoopMailer {
    async fn send(&self, to: &Email, subject: &str, _body_html: &str) -> DomainResult<()> {
        tracing::info!(to = %to.as_str(), subject, "NoopMailer: email not actually sent");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_mailer_succeeds() {
        let to = Email::new("user@example.com").unwrap();
        assert!(NoopMailer.send(&to, "Hello", "<b>Hi</b>").await.is_ok());
    }
}
