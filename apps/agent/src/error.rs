pub use anyhow::{anyhow, Error, Result};

// Define some helper functions for common error types
pub fn websocket_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Error {
    anyhow!("WebSocket error: {}", err)
}

pub fn serialization_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Error {
    anyhow!("Serialization error: {}", err)
}

pub fn command_error(msg: impl AsRef<str>) -> Error {
    anyhow!("Command execution error: {}", msg.as_ref())
}

pub fn channel_error(msg: impl AsRef<str>) -> Error {
    anyhow!("Channel error: {}", msg.as_ref())
}


pub fn url_parse_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Error {
    anyhow!("URL parsing error: {}", err)
}

pub fn io_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Error {
    anyhow!("IO error: {}", err)
}
