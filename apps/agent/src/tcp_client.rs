use crate::config::AgentConfig;
use crate::error::{self, Result};
use crate::job::JobExecutor;
use crate::state::SharedClientState;
use protocol::{AgentMessage, AgentStatus, JobMessage, Message};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

pub struct TcpClient {
    config: AgentConfig,
    state: SharedClientState,
    tx_to_server: Option<mpsc::Sender<Message>>,
    job_executor: Arc<JobExecutor>,
}

impl TcpClient {
    pub fn new(
        config: AgentConfig,
        state: SharedClientState,
        job_executor: Arc<JobExecutor>,
    ) -> Self {
        Self {
            config,
            state,
            tx_to_server: None,
            job_executor,
        }
    }

    pub fn builder() -> TcpClientBuilder {
        TcpClientBuilder::default()
    }

    async fn send_message_to_server(&self, message: Message) -> Result<()> {
        if let Some(tx) = &self.tx_to_server {
            tx.send(message)
                .await
                .map_err(|e| error::channel_error(format!("Failed to send message: {}", e)))
        } else {
            Err(error::channel_error("Send channel not initialized"))
        }
    }

    pub async fn run(mut self) -> Result<()> {
        // Connect to the TCP server
        let stream = TcpStream::connect(&self.config.tcp_url).await?;
        info!("TCP connected to {}", self.config.tcp_url);

        // Create channels for communication
        let (tx_to_socket, rx_from_app) = mpsc::channel::<Message>(self.config.channel_size);
        let (tx_from_socket, mut rx_from_socket) =
            mpsc::channel::<Message>(self.config.channel_size);

        // Channel to signal connection loss
        let (tx_connection_status, mut rx_connection_status) = mpsc::channel::<()>(1);

        self.tx_to_server = Some(tx_to_socket.clone());
        self.job_executor
            .set_tx_to_server(tx_to_socket.clone())
            .await;

        // Split the socket for reading and writing
        let (mut reader, mut writer) = tokio::io::split(stream);

        // Spawn a task to read messages from the server
        let tx_from_socket_clone = tx_from_socket.clone();
        let tx_connection_status_clone = tx_connection_status.clone();
        let _read_task = tokio::spawn(async move {
            let mut buffer = vec![0; 4096];
            let mut pending_data = String::new();

            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        // Connection closed
                        info!("Server closed the connection");
                        let _ = tx_connection_status_clone.send(()).await;
                        break;
                    }
                    Ok(n) => {
                        pending_data.push_str(&String::from_utf8_lossy(&buffer[..n]));
                        while let Some(pos) = pending_data.find("\r\n") {
                            let message_text = pending_data[..pos].to_string();
                            pending_data = pending_data[pos + 2..].to_string();
                            match protocol::serde_json::from_str::<Message>(&message_text) {
                                Ok(message) => {
                                    if tx_from_socket_clone.send(message).await.is_err() {
                                        error!("Failed to forward message from socket");
                                        let _ = tx_connection_status_clone.send(()).await;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to deserialize message: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("TCP read error: {}", e);
                        let _ = tx_connection_status_clone.send(()).await;
                        break;
                    }
                }
            }
            // If we exit the loop for any other reason, also signal connection loss
            let _ = tx_connection_status_clone.send(()).await;
        });

        // Spawn a task to write messages to the server
        let tx_connection_status_write = tx_connection_status.clone();
        let _write_task = tokio::spawn(async move {
            let mut rx = rx_from_app;

            while let Some(message) = rx.recv().await {
                match protocol::serde_json::to_string(&message) {
                    Ok(json) => {
                        debug!("Sending message: {}", json);
                        let mut data = json.into_bytes();
                        data.extend_from_slice(b"\r\n");
                        if let Err(e) = writer.write_all(&data).await {
                            error!("Failed to send on TCP: {}", e);
                            let _ = tx_connection_status_write.send(()).await;
                            break;
                        }
                        if let Err(e) = writer.flush().await {
                            error!("Failed to flush TCP: {}", e);
                            let _ = tx_connection_status_write.send(()).await;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                    }
                }
            }

            info!("Write task completed");
            // If we exit the loop for any reason, signal connection loss
            let _ = tx_connection_status_write.send(()).await;
        });

        // Main loop to process messages
        loop {
            select! {
                Some(message) = rx_from_socket.recv() => {
                    debug!("Message received: {:?}", message);
                    self.handle_message(message).await?;
                }
                Some(_) = rx_connection_status.recv() => {
                    info!("Connection lost, triggering reconnection");
                    return Err(error::anyhow!("Connection lost"));
                }
                else => {
                    info!("All channels are closed, stopping the loop");
                    break Ok(());
                }
            }
        }
    }

    async fn handle_message(&mut self, message: Message) -> Result<()> {
        match message {
            Message::Job(job_message) => match job_message {
                JobMessage::Execute { job } => {
                    let job_executor = self.job_executor.clone();
                    tokio::spawn(async move {
                        if let Err(e) = job_executor.execute_job(job).await {
                            error!("Failed to execute job: {}", e);
                        }
                    });
                }
                JobMessage::Cancel { job_id } => {
                    if let Err(e) = self.job_executor.cancel_job(job_id).await {
                        error!("Failed to cancel job: {}", e);
                    }
                }
                _ => {}
            },
            Message::Agent(agent_message) => match agent_message {
                AgentMessage::Register { agent_id } => {
                    let mut state = self.state.lock().await;
                    state.set_agent_id(agent_id);

                    let confirm = Message::Agent(AgentMessage::Heartbeat {
                        agent_id,
                        status: AgentStatus::Available,
                    });
                    self.send_message_to_server(confirm).await?;
                }
                AgentMessage::Heartbeat { .. } => {
                    unreachable!("Agent should not receive Heartbeat messages")
                }
                AgentMessage::Connected { .. } | AgentMessage::Disconnected { .. } => unreachable!(
                    "Agent client side cannot receive Connected or Disconnected messages since there are not serialized on the server side"
                ),
                AgentMessage::GetAgentsResponse { .. } => unreachable!(),
            },
            Message::Api(_) => unreachable!("Agent should not receive API messages"),
        }

        Ok(())
    }
}

pub struct TcpClientBuilder {
    config: Option<AgentConfig>,
    state: Option<SharedClientState>,
    job_executor: Option<Arc<JobExecutor>>,
}

impl Default for TcpClientBuilder {
    fn default() -> Self {
        Self {
            config: Some(AgentConfig::default()),
            state: None,
            job_executor: None,
        }
    }
}

impl TcpClientBuilder {
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn state(mut self, state: SharedClientState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn job_executor(mut self, job_executor: Arc<JobExecutor>) -> Self {
        self.job_executor = Some(job_executor);
        self
    }

    pub fn build(self) -> Result<TcpClient> {
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config is required"))?;
        let state = self.state.unwrap_or_else(crate::state::new_shared_state);

        let job_executor = if let Some(executor) = self.job_executor {
            executor
        } else {
            Arc::new(JobExecutor::new(None, state.clone()))
        };

        Ok(TcpClient::new(config, state, job_executor))
    }
}
