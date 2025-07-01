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

/// Trait for handling WebSocket connections
#[async_trait::async_trait]
pub trait WebSocketHandler {
    /// The state type used by the handler
    type State;

    /// Handle a WebSocket connection
    async fn handle_socket(socket: WebSocket, state: Self::State);

    /// Process a message received from a WebSocket
    async fn process_message(
        text: String,
        state: &Self::State,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Send a message to a WebSocket
    async fn send_message(
        message: Message,
        sender: &mut futures::stream::SplitSink<WebSocket, WsMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

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
}

#[async_trait::async_trait]
impl WebSocketHandler for AgentController {
    type State = Arc<AppState>;

    // Process WebSocket connection from an agent
    async fn handle_socket(socket: WebSocket, state: Self::State) {
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
        if let Err(e) = Self::send_message(register_msg, &mut sender).await {
            error!("Failed to send registration message: {}", e);
            Self::handle_disconnect(&state, agent_id).await;
            return;
        }

        // Task to forward messages from server to agent
        let mut send_task = tokio::spawn(async move {
            while let Some(message) = client_rx.recv().await {
                if let Err(e) = Self::send_message(message, &mut sender).await {
                    error!("Error sending message to agent: {}", e);
                    break;
                }
            }
        });

        // Task to process messages from agent
        let state_clone = state.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(result) = receiver.next().await {
                match result {
                    Ok(WsMessage::Text(text)) => {
                        if let Err(e) = Self::process_message(text.to_string(), &state_clone).await
                        {
                            error!("Error processing message: {}", e);
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

            Self::handle_disconnect(&state_clone, agent_id).await;
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

    async fn process_message(
        text: String,
        state: &Self::State,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Received message from agent: {}", text);
        let message = serde_json::from_str::<Message>(&text)?;
        state.core_tx.send(message).await?;
        Ok(())
    }

    async fn send_message(
        message: Message,
        sender: &mut futures::stream::SplitSink<WebSocket, WsMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(&message)?;
        sender.send(WsMessage::Text(json.into())).await?;
        Ok(())
    }
}

impl AgentController {
    // Helper method to handle agent disconnection
    async fn handle_disconnect(state: &Arc<AppState>, agent_id: Uuid) {
        if let Err(e) = state
            .core_tx
            .send(Message::Agent(AgentMessage::Disconnected { agent_id }))
            .await
        {
            error!("Failed to send disconnect message: {}", e);
        }
    }
}
