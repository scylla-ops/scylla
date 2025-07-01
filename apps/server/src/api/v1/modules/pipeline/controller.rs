use crate::AppState;
use crate::api::v1::common::responses::helper::ApiResponse;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use protocol::serde::Deserialize;
use protocol::{ApiMessage, Message, Pipeline};
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
pub struct CommandRequest {
    #[serde(flatten)]
    pipeline: Pipeline,
}

// Pipeline controller for handling pipeline-related HTTP requests
pub struct PipelineController;

impl PipelineController {
    // Function to execute a command
    pub async fn execute_command(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<CommandRequest>,
    ) -> impl IntoResponse {
        if let Err(e) = state
            .core_tx
            .send(Message::Api(ApiMessage::ExecutePipeline {
                pipeline: payload.pipeline,
            }))
            .await
        {
            error!("Failed to send pipeline execution message: {}", e);
            return ApiResponse::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to send pipeline execution message: {}", e),
            );
        }
        ApiResponse::success(
            StatusCode::OK,
            "Pipeline execution command sent successfully",
        )
    }
}
