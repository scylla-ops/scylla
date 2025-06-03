use crate::constants::MAX_CHANNEL_SIZE;
use crate::server::ServerError;
use crate::server_config::ServerConfig;
use protocol::{AgentMessage, Message, serde_json};
use protocol::serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::log::warn;
use tracing::{error, info};

#[derive(Debug)]
pub struct ClientConnection {
    socket: TcpStream,
    config: ServerConfig,
    client_rx: mpsc::Receiver<Message>,
    server_tx: mpsc::Sender<Message>,
}

impl ClientConnection {
    pub fn new(
        socket: TcpStream,
        config: ServerConfig,
        server_tx: mpsc::Sender<Message>,
    ) -> (Self, mpsc::Sender<Message>) {
        let (client_tx, client_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);

        (
            Self {
                socket,
                config,
                client_rx,
                server_tx,
            },
            client_tx,
        )
    }

    pub async fn process(&mut self) -> Result<(), ServerError> {
        let mut buffer = vec![0; self.config.max_buffer_size];

        loop {
            tokio::select! {
                read_result = self.socket.read(&mut buffer) => {
                    match read_result {
                        Ok(n) => {
                            if n == 0 {
                                //info!("Connexion {:?} fermée par le client", self.id);
                                break;
                            }

                            let data = &buffer[..n];
                            self.handle_message(data).await?;
                        },
                        Err(e) => {
                            warn!("Erreur de lecture depuis le socket: {}", e);
                            return Err(ServerError::Io(e));
                        }
                    }
                },

                message_opt = self.client_rx.recv() => {
                    match message_opt {
                        Some(message) => {
                            self.handle_server_message(message).await?;
                        },
                        None => {
                            info!("Canal de messages internes fermé");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_server_message(&mut self, message: Message) -> Result<(), ServerError> {
        match message {
            Message::Job(..) => self.send(message).await,
            Message::Agent(agent_message) => match agent_message {
                AgentMessage::Register { agent_id: _agent_id } => {
                    unreachable!()
                }
                AgentMessage::Heartbeat { .. } => {
                    unreachable!()
                }
            },
            Message::Api(_) => {
                unreachable!()
            }
        }
    }

    async fn handle_message(&mut self, data: &[u8]) -> Result<(), ServerError> {
        match std::str::from_utf8(data) {
            Ok(text) => {
                for line in text.split('\n') {
                    if line.trim().is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<Message>(line) {
                        Ok(message) => {
                            self.server_tx.send(message).await.unwrap();
                        }
                        Err(err) => {
                            error!("Erreur de désérialisation du message: {}", err);
                            info!("Message texte reçu: {}", line);
                        }
                    }
                }
            }
            Err(_) => {
                info!("Données binaires reçues de taille {}", data.len());
            }
        }
        Ok(())
    }

    pub async fn send<T>(&mut self, message: T) -> Result<(), ServerError>
    where
        T: Serialize + Send,
    {
        let json = serde_json::to_string(&message)
            .map_err(|e| ServerError::Serialization(e.to_string()))?;

        self.socket
            .write_all(json.as_bytes())
            .await
            .map_err(|e| ServerError::Connection(e.to_string()))?;

        Ok(())
    }
}
