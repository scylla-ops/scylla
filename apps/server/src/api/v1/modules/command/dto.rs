use serde::{Deserialize, Serialize};
use validator::Validate;

// Example DTO for a command request (similar to the existing CommandRequest)
#[derive(Deserialize, Validate)]
pub struct CommandRequestDto {
    #[validate(length(min = 1, message = "Command cannot be empty"))]
    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,
}

// Example DTO for a command response
#[derive(Serialize)]
pub struct CommandResponseDto {
    pub command_id: String,
    pub status: String,
}
