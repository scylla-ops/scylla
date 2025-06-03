use crate::agents_manager::AgentsManager;
use crate::server_config::CoreConfig;
use protocol::uuid::Uuid;
use protocol::{AgentMessage, AgentStatus, ApiMessage, HasStatus, HasUuid, JobMessage, Message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{debug, info, warn};
use anyhow::{Context, Result, anyhow};

pub struct Core {
    config: CoreConfig,
    core_rx: mpsc::Receiver<Message>,
    // Keep a core_tx for agents
    agents_manager: Arc<Mutex<AgentsManager>>,
    jobs: Arc<Mutex<HashMap<Uuid, protocol::Job>>>,
}

impl Core {
    pub fn new(
        config: CoreConfig,
        core_rx: mpsc::Receiver<Message>,
        agents_manager: Arc<Mutex<AgentsManager>>,
        jobs: Arc<Mutex<HashMap<Uuid, protocol::Job>>>,
    ) -> Self {
        Self {
            config,
            core_rx,
            agents_manager,
            jobs,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Core started on {}", self.config.addr);

        while let Some(message) = self.core_rx.recv().await {
            self.handle_message(message).await
                .with_context(|| "Failed to handle message")?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> Result<()> {
        debug!("Core received message: {:?}", message);
        match message {
            Message::Api(api_message) => match api_message {
                ApiMessage::ExecutePipeline { pipeline } => {
                    let agent_id = {
                        let agents = self.agents_manager.lock().await;
                        agents
                            .get_agent_by_filter(|agent| agent.status() == AgentStatus::Available)
                            .into_iter()
                            .next()
                            .map(|agent| agent.uuid())
                    };
                    if let Some(agent_id) = agent_id {
                        self.agents_manager
                            .lock()
                            .await
                            .update_status(agent_id, AgentStatus::Busy)
                            .with_context(|| format!("Failed to update status for agent {}", agent_id))?;
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
                        // Store the job
                        self.jobs.lock().await.insert(job_id, job.clone());
                        // Send it to the agent
                        if let Err(e) = self
                            .send_to_agent(agent_id, Message::Job(JobMessage::Execute { job }))
                            .await
                        {
                            warn!("Failed to send job to agent {}: {}", agent_id, e);
                        }
                    } else {
                        warn!("No agent available to execute the pipeline.");
                    }
                }
            },
            Message::Agent(agent_message) => match agent_message {
                AgentMessage::Register { .. } => {
                    warn!(
                        "Core received a Register message. This should be handled by the WebSocket handler."
                    );
                }
                AgentMessage::Heartbeat { agent_id, status } => {
                    self.agents_manager
                        .lock()
                        .await
                        .update_status(agent_id, status)
                        .with_context(|| format!("Failed to update status for agent {}", agent_id))?;
                    self.agents_manager.lock().await.update_last_seen(agent_id)
                        .with_context(|| format!("Failed to update last_seen for agent {}", agent_id))?;
                }
            },
            Message::Job(job_message) => match job_message {
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
                        if matches!(status, protocol::Status::Success | protocol::Status::Failed) {
                            jobs.remove(&job_id);
                        }
                    }
                }
                JobMessage::Execute { .. } => {
                    warn!("JobMessage::Execute received by core, ignored.");
                }
                JobMessage::Cancel { job_id } => {
                    self.jobs.lock().await.remove(&job_id);
                }
            },
        }
        Ok(())
    }

    pub async fn send_to_agent(&self, agent_id: Uuid, message: Message) -> Result<()> {
        let agents = self.agents_manager.lock().await;

        if let Some(tx) = agents.get_agent_tx(&agent_id) {
            tx.send(message).await
                .with_context(|| format!("Failed to send message to agent {}", agent_id))?;
            Ok(())
        } else {
            Err(anyhow!("Agent with UUID {} not found", agent_id))
        }
    }
}
