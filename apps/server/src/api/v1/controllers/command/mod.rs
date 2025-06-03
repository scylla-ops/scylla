use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info};
use validator::Validate;

use crate::api::v1::common::response_helpers;
use crate::api::v1::dto::{CommandRequestDto, CommandResponseDto};
use crate::api::v1::services::CommandService;

// Command controller for handling command-related HTTP requests
pub struct CommandController;

impl CommandController {
    // Handler for executing a command
    pub async fn execute_command(
        State(service): State<Arc<CommandService>>,
        Json(request): Json<CommandRequestDto>,
    ) -> impl IntoResponse {
        // Validate the request
        if let Err(e) = request.validate() {
            error!("Invalid command request: {}", e);
            return response_helpers::validation_error::<CommandResponseDto>(e);
        }

        // Execute the command using the service
        match service.execute_command(request).await {
            Ok(response) => {
                info!("Command executed successfully: {}", response.command_id);
                (StatusCode::OK, response_helpers::success(response))
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
                response_helpers::internal_error::<CommandResponseDto>(format!(
                    "Failed to execute command: {}",
                    e
                ))
            }
        }
    }

    // Handler for getting a command by ID
    pub async fn get_command(
        State(service): State<Arc<CommandService>>,
        Path(command_id): Path<String>,
    ) -> impl IntoResponse {
        match service.get_command(&command_id).await {
            Ok(Some(response)) => {
                info!("Command retrieved successfully: {}", command_id);
                (StatusCode::OK, response_helpers::success(response))
            }
            Ok(None) => {
                info!("Command not found: {}", command_id);
                response_helpers::not_found::<CommandResponseDto>(command_id)
            }
            Err(e) => {
                error!("Failed to get command: {}", e);
                response_helpers::internal_error::<CommandResponseDto>(format!(
                    "Failed to get command: {}",
                    e
                ))
            }
        }
    }
}
