use crate::config::AgentConfig;
use crate::error::{self, Result};
use crate::job::JobExecutor;
use crate::state::SharedClientState;
use futures::{SinkExt, StreamExt};
use protocol::{AgentMessage, AgentStatus, JobMessage, Message};
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info};
use url::Url;

pub struct WebSocketClient {
    config: AgentConfig,
    state: SharedClientState,
    tx_to_server: Option<mpsc::Sender<Message>>,
    job_executor: Arc<JobExecutor>,
}

impl WebSocketClient {
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

    pub fn builder() -> WebSocketClientBuilder {
        WebSocketClientBuilder::default()
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
        let url = Url::parse(&self.config.websocket_url)?;

        let (ws_stream, _) = connect_async(url.to_string()).await?;
        info!("WebSocket connected");

        let (tx_to_socket, rx_from_app) = mpsc::channel::<Message>(self.config.channel_size);
        let (tx_from_socket, mut rx_from_socket) =
            mpsc::channel::<Message>(self.config.channel_size);

        // Channel to signal connection loss
        let (tx_connection_status, mut rx_connection_status) = mpsc::channel::<()>(1);

        self.tx_to_server = Some(tx_to_socket.clone());
        self.job_executor
            .set_tx_to_server(tx_to_socket.clone())
            .await;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let tx_from_socket_clone = tx_from_socket.clone();
        let tx_connection_status_clone = tx_connection_status.clone();
        let _read_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        match protocol::serde_json::from_str::<Message>(&text) {
                            Ok(message) => {
                                if tx_from_socket_clone.send(message).await.is_err() {
                                    error!("Failed to forward message from socket");
                                    let _ = tx_connection_status_clone.send(()).await;
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize message: {}", e);
                                error!("message: {}", text);
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("WebSocket closed by server");
                        let _ = tx_connection_status_clone.send(()).await;
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        let _ = tx_connection_status_clone.send(()).await;
                        break;
                    }
                    _ => {}
                }
            }
            // If we exit the loop for any other reason, also signal connection loss
            let _ = tx_connection_status_clone.send(()).await;
        });

        let tx_connection_status_write = tx_connection_status.clone();
        let _write_task = tokio::spawn(async move {
            let mut rx = rx_from_app;

            while let Some(message) = rx.recv().await {
                match protocol::serde_json::to_string(&message) {
                    Ok(json) => {
                        debug!("Sending message: {}", json);
                        if let Err(e) = ws_sender.send(WsMessage::Text(json.into())).await {
                            error!("Failed to send on WebSocket: {}", e);
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
            },
            Message::Api(_) => unreachable!("Agent should not receive API messages"),
        }

        Ok(())
    }
}

pub struct WebSocketClientBuilder {
    config: Option<AgentConfig>,
    state: Option<SharedClientState>,
    job_executor: Option<Arc<JobExecutor>>,
}

impl Default for WebSocketClientBuilder {
    fn default() -> Self {
        Self {
            config: Some(AgentConfig::default()),
            state: None,
            job_executor: None,
        }
    }
}

impl WebSocketClientBuilder {
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

    pub fn build(self) -> Result<WebSocketClient> {
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config is required"))?;
        let state = self.state.unwrap_or_else(crate::state::new_shared_state);

        let job_executor = if let Some(executor) = self.job_executor {
            executor
        } else {
            Arc::new(JobExecutor::new(None, state.clone()))
        };

        Ok(WebSocketClient::new(config, state, job_executor))
    }
}
