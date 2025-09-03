use crate::AppState;
use crate::config::CoreConfig;
use anyhow::{Context, Result};
use protocol::uuid::Uuid;
use protocol::{AgentMessage, Message};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// TCP server for agent connections
pub struct TcpServer {
    listener: TcpListener,
    app_state: Arc<AppState>,
}

impl TcpServer {
    /// Create a new TCP server
    pub async fn new(addr: &str, app_state: Arc<AppState>) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind TCP server to {addr}"))?;

        info!("TCP server for agents listening on {}", addr);

        Ok(Self {
            listener,
            app_state,
        })
    }

    pub async fn spawn_tcp_server(
        core_config: &CoreConfig,
        app_state: Arc<AppState>,
    ) -> JoinHandle<()> {
        let tcp_server = TcpServer::new(&core_config.tcp_addr.to_string(), app_state).await;

        let tcp_task = tokio::spawn(async move {
            match tcp_server {
                Ok(tcp) => {
                    tcp.run().await.unwrap();
                }
                Err(e) => {
                    error!("TCP server error: {e:#}");
                }
            }
        });

        info!("TCP server for agents running on {}", core_config.tcp_addr);
        tcp_task
    }

    /// Run the TCP server
    pub async fn run(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New agent connection from {}", addr);
                    let app_state = self.app_state.clone();

                    // Spawn a new task to handle this connection
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(socket, app_state).await {
                            error!("Error handling agent connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a new agent connection
    async fn handle_connection(socket: TcpStream, state: Arc<AppState>) -> Result<()> {
        // Create a new UUID for this agent
        let agent_id = Uuid::new_v4();

        info!("New agent connected with ID: {}", agent_id);

        // Create channels for communication
        let (client_tx, mut client_rx) = mpsc::channel::<Message>(crate::config::MAX_CHANNEL_SIZE);

        // Split the socket for reading and writing
        let (reader, mut writer) = tokio::io::split(socket);

        // Notify core about new agent connection
        if let Err(e) = state
            .core_tx
            .send(Message::Agent(AgentMessage::Connected {
                agent_id,
                tx: client_tx,
            }))
            .await
        {
            error!("Failed to notify core about agent connection: {}", e);
            return Err(e.into());
        }

        // Send registration message to the agent
        let register_msg = Message::Agent(AgentMessage::Register { agent_id });
        if let Err(e) = Self::send_message(&mut writer, register_msg).await {
            error!("Failed to send registration message: {}", e);
            Self::handle_disconnect(&state, agent_id).await?;
            return Err(e);
        }

        // Spawn a task to read messages from the agent
        let state_clone = state.clone();
        let read_task = tokio::spawn(async move {
            let mut buf_reader = BufReader::new(reader);
            let mut buffer = Vec::new();

            loop {
                buffer.clear();
                match buf_reader.read_until(b'\n', &mut buffer).await {
                    Ok(0) => {
                        // Connection closed
                        info!("Agent {} disconnected", agent_id);
                        break;
                    }
                    Ok(_) => {
                        // Remove trailing \r\n or \n Sus
                        if buffer.ends_with(b"\r\n") {
                            buffer.truncate(buffer.len() - 2);
                        } else if buffer.ends_with(b"\n") {
                            buffer.truncate(buffer.len() - 1);
                        }

                        if let Err(e) = Self::process_message(&buffer, &state_clone, agent_id).await
                        {
                            error!("Error processing message from agent {}: {}", agent_id, e);
                        }
                    }
                    Err(e) => {
                        error!("Error reading from agent {}: {}", agent_id, e);
                        break;
                    }
                }
            }

            // Handle disconnection
            if let Err(e) = Self::handle_disconnect(&state_clone, agent_id).await {
                error!("Error handling disconnect for agent {}: {}", agent_id, e);
            }
        });

        // Task to forward messages from server to agent
        let write_task = tokio::spawn(async move {
            while let Some(message) = client_rx.recv().await {
                if let Err(e) = Self::send_message(&mut writer, message).await {
                    error!("Error sending message to agent {}: {}", agent_id, e);
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = read_task => {},
            _ = write_task => {},
        }

        Ok(())
    }

    /// Process a message received from an agent
    async fn process_message(
        message_bytes: &[u8],
        state: &Arc<AppState>,
        agent_id: Uuid,
    ) -> Result<()> {
        let text = String::from_utf8(message_bytes.to_vec())?;
        debug!("Received message from agent {}: {}", agent_id, text);

        let message = serde_json::from_str::<Message>(&text)?;
        state.core_tx.send(message).await?;

        Ok(())
    }

    /// Send a message to an agent
    async fn send_message(
        writer: &mut tokio::io::WriteHalf<TcpStream>,
        message: Message,
    ) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        let data = json.as_bytes();

        writer.write_all(data).await?;
        writer.write_all(b"\r\n").await?;
        writer.flush().await?;

        Ok(())
    }

    /// Handle agent disconnection
    async fn handle_disconnect(state: &Arc<AppState>, agent_id: Uuid) -> Result<()> {
        state
            .core_tx
            .send(Message::Agent(AgentMessage::Disconnected { agent_id }))
            .await
            .map_err(|e| e.into())
    }
}
