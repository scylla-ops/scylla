use crate::command::{CommandExecutor, ShellCommandExecutor};
use crate::error::{self, Result};
use crate::state::SharedClientState;
use protocol::uuid::Uuid;
use protocol::{AgentMessage, AgentStatus, JobMessage, Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, watch};
use tracing::{debug, error};

pub struct JobExecutor {
    command_executor: Arc<dyn CommandExecutor>,
    cancel_map: Arc<Mutex<HashMap<Uuid, watch::Sender<bool>>>>,
    tx_to_server: Arc<Mutex<Option<mpsc::Sender<Message>>>>,
    state: SharedClientState,
}

impl JobExecutor {
    pub fn new(
        command_executor: Option<Arc<dyn CommandExecutor>>,
        state: SharedClientState,
    ) -> Self {
        Self {
            command_executor: command_executor
                .unwrap_or_else(|| Arc::new(ShellCommandExecutor::default())),
            cancel_map: Arc::new(Mutex::new(HashMap::new())),
            tx_to_server: Arc::new(Mutex::new(None)),
            state,
        }
    }

    pub async fn set_tx_to_server(&self, tx: mpsc::Sender<Message>) {
        let mut tx_guard = self.tx_to_server.lock().await;
        *tx_guard = Some(tx);
    }

    async fn execute_step(&self, step: protocol::JobStep) -> (protocol::Status, String) {
        let mut output_acc = String::new();
        for cmd in step.commands {
            match self.command_executor.execute(&cmd).await {
                Ok(out) => {
                    output_acc.push_str(&out);
                }
                Err(err) => {
                    let err_msg = format!("{}", err);
                    output_acc.push_str(&err_msg);
                    return (protocol::Status::Failed, output_acc);
                }
            }
        }
        (protocol::Status::Success, output_acc)
    }

    pub async fn execute_job(&self, job: protocol::Job) -> Result<()> {
        let job_id = job.id;
        let steps = job.steps;
        let mut job_status = protocol::Status::Success;

        // Update agent status to busy
        {
            let mut state = self.state.lock().await;
            state.set_status(AgentStatus::Busy);
        }

        // Set up cancellation channel
        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.cancel_map.lock().await.insert(job_id, cancel_tx);

        // Get the sender with retries
        let tx = {
            let mut retries = 0;
            const MAX_RETRIES: usize = 5;
            const RETRY_DELAY_MS: u64 = 500;

            loop {
                let tx_guard = self.tx_to_server.lock().await;
                match &*tx_guard {
                    Some(tx) => break tx.clone(),
                    None => {
                        if retries >= MAX_RETRIES {
                            error!(
                                "No sender available to communicate with server after {} retries",
                                MAX_RETRIES
                            );
                            return Err(error::channel_error("No sender available"));
                        }
                        drop(tx_guard); // Release the lock before sleeping
                        retries += 1;
                        debug!(
                            "No sender available, retrying ({}/{})",
                            retries, MAX_RETRIES
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS))
                            .await;
                    }
                }
            }
        };

        // Execute each step
        for (i, step) in steps.into_iter().enumerate() {
            if *cancel_rx.borrow() {
                job_status = protocol::Status::Failed;
                break;
            }

            // Send step running status
            if let Err(e) = tx
                .send(Message::Job(JobMessage::StepStatusUpdate {
                    job_id,
                    step_index: i,
                    status: protocol::Status::Running,
                    output: String::new(),
                }))
                .await
            {
                error!("Failed to send step status update: {}", e);
                return Err(error::channel_error(format!(
                    "Failed to send message: {}",
                    e
                )));
            }

            // Execute the step
            let (step_status, output_acc) = self.execute_step(step).await;
            if step_status == protocol::Status::Failed {
                job_status = protocol::Status::Failed;
            }

            // Send step completion status
            if let Err(e) = tx
                .send(Message::Job(JobMessage::StepStatusUpdate {
                    job_id,
                    step_index: i,
                    status: step_status,
                    output: output_acc,
                }))
                .await
            {
                error!("Failed to send step status update: {}", e);
                return Err(error::channel_error(format!(
                    "Failed to send message: {}",
                    e
                )));
            }

            if job_status == protocol::Status::Failed {
                break;
            }
        }

        // Send job completion status
        if let Err(e) = tx
            .send(Message::Job(JobMessage::JobStatusUpdate {
                job_id,
                status: job_status,
            }))
            .await
        {
            error!("Failed to send job status update: {}", e);
            return Err(error::channel_error(format!(
                "Failed to send message: {}",
                e
            )));
        }

        // Update agent status to available
        {
            let mut state = self.state.lock().await;
            state.set_status(AgentStatus::Available);
        }

        // Send heartbeat
        let agent_id = {
            let state = self.state.lock().await;
            state
                .agent_id
                .ok_or_else(|| anyhow::anyhow!("Agent ID not set"))?
        };

        if let Err(e) = tx
            .send(Message::Agent(AgentMessage::Heartbeat {
                agent_id,
                status: AgentStatus::Available,
            }))
            .await
        {
            error!("Failed to send heartbeat: {}", e);
            return Err(error::channel_error(format!(
                "Failed to send message: {}",
                e
            )));
        }

        // Remove job from cancel map
        self.cancel_map.lock().await.remove(&job_id);

        Ok(())
    }

    pub async fn cancel_job(&self, job_id: Uuid) -> Result<()> {
        let map = self.cancel_map.lock().await;
        if let Some(sender) = map.get(&job_id) {
            sender
                .send(true)
                .map_err(|e| error::channel_error(format!("Failed to send cancel signal: {}", e)))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job {} not found", job_id))
        }
    }
}
