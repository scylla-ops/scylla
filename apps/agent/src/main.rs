mod constants;

use protocol::AgentStatus::Undefined;
use protocol::{AgentMessage, AgentStatus, JobMessage, Message, serde_json};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::select;
use tokio::sync::{Mutex, mpsc, watch};
use tracing::{debug, error, info};
use protocol::uuid::Uuid;
use crate::constants::MAX_BUFFER_SIZE;

pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn execute(cmd: &str) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

struct ClientState {
    agent_id: Option<Uuid>,
    agent_status: AgentStatus,
}

impl ClientState {
    fn new() -> Self {
        Self {
            agent_id: None,
            agent_status: Undefined,
        }
    }

    fn set_agent_id(&mut self, agent_id: Uuid) {
        debug!("ID assigné: {}", agent_id);
        self.agent_id = Some(agent_id);
    }

    fn set_agent_status(&mut self, status: AgentStatus) {
        self.agent_status = status;
    }
}

struct TcpClient {
    addr: String,
    connection: Option<TcpStream>,
    state: ClientState,
    tx_to_server: Option<mpsc::Sender<Message>>,
    cancel_map: Arc<Mutex<HashMap<Uuid, watch::Sender<bool>>>>,
}

impl TcpClient {
    fn new(addr: String) -> Self {
        Self {
            addr,
            connection: None,
            state: ClientState::new(),
            tx_to_server: None,
            cancel_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Connexion à {}", self.addr);
        let connection = TcpStream::connect(&self.addr).await?;
        self.connection = Some(connection);
        Ok(())
    }

    async fn send_message_to_server(&self, message: Message) -> Result<(), Box<dyn Error>> {
        if let Some(tx) = &self.tx_to_server {
            tx.send(message)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        } else {
            Err("Canal d'envoi non initialisé".into())
        }
    }

    async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let connection = match self.connection.take() {
            Some(conn) => conn,
            None => return Err("Non connecté".into()),
        };

        let (tx_to_socket, rx_from_app) = mpsc::channel::<Message>(100);
        let (tx_from_socket, mut rx_from_socket) = mpsc::channel::<Message>(100);

        self.tx_to_server = Some(tx_to_socket.clone());

        let (mut reader, mut writer) = connection.into_split();

        let _read_task = tokio::spawn(async move {
            let mut buffer = vec![0u8; MAX_BUFFER_SIZE];
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        info!("Connexion fermée par le serveur");
                        break;
                    }
                    Ok(n) => match serde_json::from_slice::<Message>(&buffer[..n]) {
                        Ok(message) => {
                            if tx_from_socket.send(message).await.is_err() {
                                error!("Échec de transmission du message depuis la socket");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Échec de désérialisation du message: {}", e);
                            error!("message: {}", String::from_utf8_lossy(&buffer[..n]));
                        }
                    },
                    Err(e) => {
                        error!("Échec de lecture depuis la socket: {}", e);
                        break;
                    }
                }
            }
        });

        let _write_task = tokio::spawn(async move {
            let mut rx = rx_from_app;

            while let Some(message) = rx.recv().await {
                match serde_json::to_vec(&message) {
                    Ok(mut data) => {
                        debug!("envoie de la string: {}", String::from_utf8_lossy(&data));
                        data.push(b'\n'); // Ajoute un séparateur de message (\n) après chaque JSON
                        if let Err(e) = writer.write_all(&data).await {
                            error!("Échec d'écriture sur la socket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Échec de sérialisation du message: {}", e);
                    }
                }
            }

            info!("Tâche d'écriture terminée");
        });

        loop {
            select! {
                Some(message) = rx_from_socket.recv() => {
                    debug!("Message reçu: {:?}", message);
                    self.handle_message(message).await?;
                }
                else => {
                    info!("Tous les canaux sont fermés, arrêt de la boucle");
                    break Ok(());
                }
            }
        }
    }

    async fn execute_step(step: protocol::JobStep) -> (protocol::Status, String) {
        let mut output_acc = String::new();
        for cmd in step.commands {
            match CommandExecutor::execute(&cmd).await {
                Ok(out) => {
                    output_acc.push_str(&out);
                }
                Err(err) => {
                    output_acc.push_str(&err);
                    return (protocol::Status::Failed, output_acc);
                }
            }
        }
        (protocol::Status::Success, output_acc)
    }

    async fn execute_job(
        job: protocol::Job,
        tx: mpsc::Sender<Message>,
        agent_id: Uuid,
        cancel_map: Arc<Mutex<HashMap<Uuid, watch::Sender<bool>>>>,
    ) {
        let job_id = job.id;
        let steps = job.steps;
        let mut job_status = protocol::Status::Success;
        let (cancel_tx, cancel_rx) = watch::channel(false);
        cancel_map.lock().await.insert(job_id, cancel_tx);
        for (i, step) in steps.into_iter().enumerate() {
            if *cancel_rx.borrow() {
                job_status = protocol::Status::Failed;
                break;
            }
            let _ = tx
                .send(Message::Job(JobMessage::StepStatusUpdate {
                    job_id,
                    step_index: i,
                    status: protocol::Status::Running,
                    output: String::new(),
                }))
                .await;
            let (step_status, output_acc) = Self::execute_step(step).await;
            if step_status == protocol::Status::Failed {
                job_status = protocol::Status::Failed;
            }
            let _ = tx
                .send(Message::Job(JobMessage::StepStatusUpdate {
                    job_id,
                    step_index: i,
                    status: step_status.clone(),
                    output: output_acc,
                }))
                .await;
            if step_status == protocol::Status::Failed {
                break;
            }
        }
        let _ = tx
            .send(Message::Job(JobMessage::JobStatusUpdate {
                job_id,
                status: job_status,
            }))
            .await;
        let _ = tx
            .send(Message::Agent(AgentMessage::Heartbeat {
                agent_id,
                status: AgentStatus::Available,
            }))
            .await;
        cancel_map.lock().await.remove(&job_id);
    }

    async fn handle_message(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        match message {
            Message::Job(job_message) => match job_message {
                JobMessage::Execute { job } => {
                    let tx = self.tx_to_server.as_ref().unwrap().clone();
                    let agent_id = self.state.agent_id.unwrap();
                    let cancel_map = self.cancel_map.clone();
                    tokio::spawn(async move {
                        TcpClient::execute_job(job, tx, agent_id, cancel_map).await;
                    });
                }
                JobMessage::Cancel { job_id } => {
                    let map = self.cancel_map.lock().await;
                    if let Some(sender) = map.get(&job_id) {
                        let _ = sender.send(true);
                    }
                }
                _ => {}
            },
            Message::Agent(agent_message) => match agent_message {
                AgentMessage::Register { agent_id } => {
                    self.state.set_agent_id(agent_id);

                    let confirm = Message::Agent(AgentMessage::Heartbeat {
                        agent_id,
                        status: AgentStatus::Available,
                    });
                    self.send_message_to_server(confirm).await?;
                }
                AgentMessage::Heartbeat { .. } => {
                    unreachable!()
                }
            },
            Message::Api(_) => unreachable!(),
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let mut client = TcpClient::new("127.0.0.1:8080".to_string());
    client.connect().await?;

    client.run().await?;

    Ok(())
}
