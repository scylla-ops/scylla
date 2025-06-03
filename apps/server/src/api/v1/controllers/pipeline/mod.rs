use axum::Json;
use axum::extract::State;
use protocol::serde::Deserialize;
use protocol::{ApiMessage, Message, Pipeline};
use std::sync::Arc;
use tracing::error;

use crate::AppState;

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
    ) -> String {
        if let Err(e) = state
            .core_tx
            .send(Message::Api(ApiMessage::ExecutePipeline {
                pipeline: payload.pipeline,
            }))
            .await
        {
            error!("Failed to send pipeline execution message: {}", e);
            return format!("Error: {}", e);
        }

        "Ok".to_string()
    }
}
