use anyhow::{Context, Result};
use tracing::info;
use validator::Validate;

use crate::api::v1::dto::{CommandRequestDto, CommandResponseDto};
use crate::api::v1::repositories::CommandRepository;

// Command service for handling command-related business logic
pub struct CommandService {
    repository: CommandRepository,
}

impl CommandService {
    pub fn new(repository: CommandRepository) -> Self {
        Self { repository }
    }

    // Execute a command and save it to the database
    pub async fn execute_command(&self, request: CommandRequestDto) -> Result<CommandResponseDto> {
        // Validate the request
        request.validate().context("Invalid command request")?;

        // Save the command to the database
        let command_id = self
            .repository
            .save_command(&request.command, &request.args)
            .await
            .context("Failed to save command")?;

        // In a real application, you would execute the command here
        // For now, we'll just log it
        info!(
            "Executing command: {} with args: {:?}",
            request.command, request.args
        );

        // Return the response
        Ok(CommandResponseDto {
            command_id,
            status: "submitted".to_string(),
        })
    }

    // Get a command by ID
    pub async fn get_command(&self, command_id: &str) -> Result<Option<CommandResponseDto>> {
        let result = self
            .repository
            .get_command(command_id)
            .await
            .context("Failed to get command")?;

        match result {
            Some((command, args)) => {
                info!("Retrieved command: {} with args: {:?}", command, args);
                Ok(Some(CommandResponseDto {
                    command_id: command_id.to_string(),
                    status: "completed".to_string(), // In a real app, you'd get the actual status
                }))
            }
            None => Ok(None),
        }
    }
}
