use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::command::CommandService;
use crate::api::v1::modules::command::dto::CommandRequestDto;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info};
use validator::Validate;

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
            return ApiResponse::error(
                StatusCode::BAD_REQUEST,
                format!("Invalid command request: {}", e),
            );
        }

        // Execute the command using the service
        match service.execute_command(request).await {
            Ok(response) => {
                info!("Command executed successfully: {}", response.command_id);
                ApiResponse::success(StatusCode::OK, response)
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to execute command: {}", e),
                )
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
                ApiResponse::success(StatusCode::OK, response)
            }
            Ok(None) => {
                info!("Command not found: {}", command_id);
                ApiResponse::error(
                    StatusCode::NOT_FOUND,
                    format!("Command not found: {}", command_id),
                )
            }
            Err(e) => {
                error!("Failed to get command: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get command: {}", e),
                )
            }
        }
    }
}
