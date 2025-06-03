use axum::extract::ws::{Message as WsMessage, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use protocol::serde_json;
use protocol::uuid::Uuid;
use protocol::{AgentMessage, Message};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::AppState;
use crate::config::MAX_CHANNEL_SIZE;

// Agent controller for handling agent-related HTTP requests
pub struct AgentController;

impl AgentController {
    // Handler for WebSocket connections from agents
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(state): State<Arc<AppState>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(|socket| Self::handle_socket(socket, state))
    }

    // Process WebSocket connection from an agent
    async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
        let (mut sender, mut receiver) = socket.split();

        // Create a new UUID for this agent
        let agent_id = Uuid::new_v4();

        // Create channels for communication
        let (client_tx, mut client_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

        state
            .core_tx
            .send(Message::Agent(AgentMessage::Connected {
                agent_id,
                tx: client_tx,
            }))
            .await
            .unwrap();

        // Send registration message to the agent
        let register_msg = Message::Agent(AgentMessage::Register { agent_id });
        if let Ok(json) = serde_json::to_string(&register_msg) {
            if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                error!("Failed to send registration message: {}", e);
                state
                    .core_tx
                    .send(Message::Agent(AgentMessage::Disconnected { agent_id }))
                    .await
                    .unwrap();
                return;
            }
        }

        // Task to forward messages from server to agent
        let mut send_task = tokio::spawn(async move {
            while let Some(message) = client_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&message) {
                    if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                        error!("Error sending message to agent: {}", e);
                        break;
                    }
                }
            }
        });

        // Task to process messages from agent
        let core_tx = state.core_tx.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(result) = receiver.next().await {
                match result {
                    Ok(WsMessage::Text(text)) => {
                        debug!("Received message from agent: {}", text);
                        match serde_json::from_str::<Message>(&text) {
                            Ok(message) => {
                                if let Err(e) = core_tx.send(message).await {
                                    error!("Failed to forward message to core: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse message from agent: {}", e);
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("Agent disconnected");
                        break;
                    }
                    Err(e) => {
                        error!("Error receiving message from agent: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            state
                .core_tx
                .send(Message::Agent(AgentMessage::Disconnected { agent_id }))
                .await
                .unwrap();
        });

        // Wait for either task to complete
        tokio::select! {
            _ = &mut send_task => {
                recv_task.abort();
            }
            _ = &mut recv_task => {
                send_task.abort();
            }
        }
    }
}
