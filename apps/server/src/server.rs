use crate::agents_manager::AgentsManager;
use crate::client_connection::ClientConnection;
use crate::constants::MAX_CHANNEL_SIZE;
use crate::server_config::ServerConfig;
use protocol::uuid::Uuid;
use protocol::{AgentMessage, AgentStatus, ApiMessage, HasStatus, HasUuid, JobMessage, Message};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Erreur d'E/S: {0}")]
    Io(#[from] io::Error),

    #[error("Erreur de format d'adresse: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Error during serialization: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

pub struct Server {
    config: ServerConfig,
    tcp_listener: TcpListener,
    server_rx: mpsc::Receiver<Message>,
    // Keep a server_tx for agents threads
    server_tx: mpsc::Sender<Message>,
    agents_manager: Arc<Mutex<AgentsManager>>,
    jobs: Arc<Mutex<HashMap<Uuid, protocol::Job>>>,
}

impl Server {
    pub async fn with_config(
        config: ServerConfig,
    ) -> Result<(Self, mpsc::Sender<Message>), ServerError> {
        let tcp_listener = TcpListener::bind(config.addr).await?;
        let (server_tx, server_rx) = mpsc::channel::<Message>(MAX_CHANNEL_SIZE);
        let agents_manager = Arc::new(Mutex::new(AgentsManager::new()));
        let jobs = Arc::new(Mutex::new(HashMap::new()));

        Ok((
            Self {
                config,
                tcp_listener,
                server_rx,
                server_tx: server_tx.clone(),
                agents_manager,
                jobs,
            },
            server_tx,
        ))
    }

    async fn accept_client(&mut self, addr: std::net::SocketAddr, socket: tokio::net::TcpStream) {
        info!("Nouvelle connexion depuis : {}", addr);

        let (mut client, client_tx) =
            ClientConnection::new(socket, self.config.clone(), self.server_tx.clone());

        let new_uuid = Uuid::new_v4();

        if let Err(e) = client
            .send(Message::Agent(AgentMessage::Register {
                agent_id: new_uuid,
            }))
            .await
        {
            warn!(
                "Erreur lors de l'envoi de l'identifiant de l'agent {}: {}",
                new_uuid, e
            );
            return;
        }

        self.agents_manager
            .lock()
            .await
            .add_agent(new_uuid, client_tx);

        let agents_manager = self.agents_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = client.process().await {
                warn!("Erreur lors du traitement de la connexion {}: {}", addr, e);
            }
            agents_manager.lock().await.remove_agent(new_uuid);
        });
    }

    pub async fn run(&mut self) -> Result<(), ServerError> {
        info!(
            "Serveur démarré sur {}:{}",
            self.config.addr.ip(),
            self.config.addr.port()
        );

        loop {
            tokio::select! {
                accept_result = self.tcp_listener.accept() => {
                    match accept_result {
                        Ok((socket, addr)) => {
                            self.accept_client(addr, socket).await;
                        },
                        Err(e) => {
                            warn!("Erreur lors de l'acceptation d'une connexion: {}", e);
                            return Err(ServerError::Io(e));
                        }
                    }
                },

                server_msg = self.server_rx.recv() => {
                    self.handle_server_message(server_msg).await?;
                }
            }
        }
    }

    async fn handle_server_message(
        &mut self,
        server_msg: Option<Message>,
    ) -> Result<(), ServerError> {
        match server_msg {
            Some(message) => {
                debug!("Server received message: {:?}", message);
                match message {
                    Message::Api(api_message) => match api_message {
                        ApiMessage::ExecutePipeline { pipeline } => {
                            let agent_id = {
                                let agents = self.agents_manager.lock().await;
                                agents
                                    .get_agent_by_filter(|agent| {
                                        agent.status() == AgentStatus::Available
                                    })
                                    .into_iter()
                                    .next()
                                    .map(|agent| agent.uuid())
                            };
                            if let Some(agent_id) = agent_id {
                                self.agents_manager
                                    .lock()
                                    .await
                                    .update_status(agent_id, AgentStatus::Busy);
                                let job_id = Uuid::new_v4();
                                let job = protocol::Job {
                                    id: job_id,
                                    pipeline_id: pipeline.id,
                                    status: protocol::Status::Pending,
                                    steps: pipeline
                                        .steps
                                        .iter()
                                        .map(|step| protocol::JobStep {
                                            name: step.name.clone(),
                                            commands: step.commands.clone(),
                                            status: protocol::Status::Pending,
                                            output: None,
                                        })
                                        .collect(),
                                };
                                // Stocker le job
                                self.jobs.lock().await.insert(job_id, job.clone());
                                // Envoyer à l'agent
                                if let Err(e) = self
                                    .send_to_agent(
                                        agent_id,
                                        Message::Job(JobMessage::Execute { job }),
                                    )
                                    .await
                                {
                                    warn!("Failed to send job to agent {}: {}", agent_id, e);
                                }
                            } else {
                                warn!("Aucun agent disponible pour exécuter la pipeline.");
                            }
                        }
                    },
                    Message::Agent(agent_message) => match agent_message {
                        AgentMessage::Register { .. } => {
                            unreachable!(
                                "Server received a Register message. This should never happen."
                            )
                        }
                        AgentMessage::Heartbeat { agent_id, status } => {
                            self.agents_manager
                                .lock()
                                .await
                                .update_status(agent_id, status);
                            self.agents_manager.lock().await.update_last_seen(agent_id);
                        }
                    },
                    Message::Job(job_message) => {
                        match job_message {
                            JobMessage::StepStatusUpdate {
                                job_id,
                                step_index,
                                status,
                                output,
                            } => {
                                let mut jobs = self.jobs.lock().await;
                                if let Some(job) = jobs.get_mut(&job_id) {
                                    if let Some(step) = job.steps.get_mut(step_index) {
                                        step.status = status;
                                        step.output = Some(output);
                                    }
                                }
                            }
                            JobMessage::JobStatusUpdate { job_id, status } => {
                                let mut jobs = self.jobs.lock().await;
                                if let Some(job) = jobs.get_mut(&job_id) {
                                    job.status = status.clone();
                                    if matches!(
                                        status,
                                        protocol::Status::Success | protocol::Status::Failed
                                    ) {
                                        jobs.remove(&job_id);
                                    }
                                }
                            }
                            JobMessage::Execute { .. } => {
                                // Le serveur ne doit pas recevoir ce message
                                warn!("JobMessage::Execute reçu côté serveur, ignoré.");
                            }
                            JobMessage::Cancel { job_id } => {
                                // À implémenter si besoin
                                self.jobs.lock().await.remove(&job_id);
                            }
                        }
                    }
                }
            }
            None => {
                unreachable!("Server received a None message. This should never happen.")
            }
        }
        Ok(())
    }

    pub async fn send_to_agent(&self, agent_id: Uuid, message: Message) -> Result<(), ServerError> {
        let agents = self.agents_manager.lock().await;

        if let Some(tx) = agents.get_agent_tx(&agent_id) {
            tx.send(message).await.map_err(|_| {
                ServerError::Connection(format!(
                    "Échec de l'envoi du message à l'agent {}",
                    agent_id
                ))
            })?;
            Ok(())
        } else {
            Err(ServerError::Connection(format!(
                "Agent avec UUID {} non trouvé",
                agent_id
            )))
        }
    }
}
