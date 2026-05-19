use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListenerError {
    #[error("failed to create broker subscriber: {0}")]
    SubscriberInit(String),
    #[error("failed to subscribe to '{subject}': {message}")]
    Subscribe { subject: String, message: String },
}
