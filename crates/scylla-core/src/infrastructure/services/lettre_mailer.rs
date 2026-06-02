use crate::application::Mailer;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::Email;
use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// SMTP mailer over an implicitly-TLS relay (port from config). Behind the
/// `mail` feature so PaaS builds don't pull in `lettre`.
pub struct LettreMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl LettreMailer {
    pub fn new(
        host: &str,
        port: u16,
        username: String,
        password: String,
        from: &str,
    ) -> DomainResult<Self> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| DomainError::infrastructure(format!("smtp relay setup: {e}")))?
            .port(port)
            .credentials(Credentials::new(username, password))
            .build();
        let from = from
            .parse::<Mailbox>()
            .map_err(|e| DomainError::infrastructure(format!("invalid 'from' address: {e}")))?;
        Ok(Self { transport, from })
    }
}

#[async_trait]
impl Mailer for LettreMailer {
    async fn send(&self, to: &Email, subject: &str, body_html: &str) -> DomainResult<()> {
        let to: Mailbox = to
            .as_str()
            .parse()
            .map_err(|e| DomainError::validation(format!("invalid recipient address: {e}")))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body_html.to_string())
            .map_err(|e| DomainError::internal(format!("build email: {e}")))?;
        self.transport
            .send(message)
            .await
            .map_err(|e| DomainError::infrastructure(format!("smtp send: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_from_address() {
        let res = LettreMailer::new(
            "smtp.example.com",
            465,
            "u".into(),
            "p".into(),
            "not-an-email",
        );
        assert!(res.is_err());
    }

    #[test]
    fn builds_with_valid_config() {
        let res = LettreMailer::new(
            "smtp.example.com",
            465,
            "user".into(),
            "pass".into(),
            "Scylla <no-reply@scylla.dev>",
        );
        assert!(res.is_ok());
    }
}
