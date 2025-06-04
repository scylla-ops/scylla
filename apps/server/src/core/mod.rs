use crate::agents::AgentsManager;
use crate::config::CoreConfig;
use anyhow::{Context, Result, anyhow};
use protocol::uuid::Uuid;
use protocol::{AgentMessage, AgentStatus, ApiMessage, HasStatus, HasUuid, JobMessage, Message};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Trait for handling different types of messages
pub trait MessageHandler {
    /// Handle a message and return a result
    async fn handle_message(&mut self, message: Message) -> Result<()>;
}

/// Trait for sending messages to agents
pub trait AgentSender {
    /// Send a message to an agent
    async fn send_to_agent(&self, agent_id: Uuid, message: Message) -> Result<()>;
}

pub struct Core {
    config: CoreConfig,
    core_rx: mpsc::Receiver<Message>,
    agents_manager: AgentsManager,
    jobs: HashMap<Uuid, protocol::Job>,
}

impl Core {
    pub fn new(config: CoreConfig, core_rx: mpsc::Receiver<Message>) -> Self {
        Self {
            config,
            core_rx,
            agents_manager: AgentsManager::new(),
            jobs: HashMap::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Core started on {}", self.config.addr);

        while let Some(message) = self.core_rx.recv().await {
            self.handle_message(message)
                .await
                .with_context(|| "Failed to handle message")?;
        }

        Ok(())
    }

    async fn handle_api_message(&mut self, api_message: ApiMessage) -> Result<()> {
        match api_message {
            ApiMessage::ExecutePipeline { pipeline } => {
                self.handle_execute_pipeline(pipeline).await?;
            }
        }
        Ok(())
    }

    async fn handle_execute_pipeline(&mut self, pipeline: protocol::Pipeline) -> Result<()> {
        let agent_id = self.find_available_agent();

        if let Some(agent_id) = agent_id {
            self.agents_manager
                .update_status(agent_id, AgentStatus::Busy)
                .map_err(|e| anyhow!("Failed to update status for agent {}: {:?}", agent_id, e))?;

            let job = self.create_job_from_pipeline(pipeline);
            let job_id = job.id;

            // Store the job
            self.jobs.insert(job_id, job.clone());

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

        Ok(())
    }

    fn find_available_agent(&self) -> Option<Uuid> {
        let agents = &self.agents_manager;
        agents
            .get_agent_by_filter(|agent| agent.status() == AgentStatus::Available)
            .into_iter()
            .next()
            .map(|agent| agent.uuid())
    }

    fn create_job_from_pipeline(&self, pipeline: protocol::Pipeline) -> protocol::Job {
        let job_id = Uuid::new_v4();
        protocol::Job {
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
        }
    }

    async fn handle_agent_message(&mut self, agent_message: AgentMessage) -> Result<()> {
        match agent_message {
            AgentMessage::Register { .. } => {
                warn!(
                    "Core received a Register message. This should be handled by the WebSocket handler."
                );
            }
            AgentMessage::Heartbeat { agent_id, status } => {
                self.agents_manager
                    .update_status(agent_id, status)
                    .map_err(|e| {
                        anyhow!("Failed to update status for agent {}: {:?}", agent_id, e)
                    })?;
                self.agents_manager
                    .update_last_seen(agent_id)
                    .map_err(|e| {
                        anyhow!("Failed to update last_seen for agent {}: {:?}", agent_id, e)
                    })?;
            }
            AgentMessage::Connected { agent_id, tx } => {
                self.agents_manager.add_agent(agent_id, tx);
                info!("Agent {} connected", agent_id);
            }
            AgentMessage::Disconnected { agent_id } => {
                self.agents_manager.remove_agent(agent_id);
                info!("Agent {} disconnected", agent_id);
            }
        }
        Ok(())
    }

    async fn handle_job_message(&mut self, job_message: JobMessage) -> Result<()> {
        match job_message {
            JobMessage::StepStatusUpdate {
                job_id,
                step_index,
                status,
                output,
            } => {
                self.update_job_step(job_id, step_index, status, output);
            }
            JobMessage::JobStatusUpdate { job_id, status } => {
                self.update_job_status(job_id, status);
            }
            JobMessage::Execute { .. } => {
                warn!("JobMessage::Execute received by core, ignored.");
            }
            JobMessage::Cancel { job_id } => {
                self.jobs.remove(&job_id);
            }
        }
        Ok(())
    }

    fn update_job_step(
        &mut self,
        job_id: Uuid,
        step_index: usize,
        status: protocol::Status,
        output: String,
    ) {
        let jobs = &mut self.jobs;
        if let Some(job) = jobs.get_mut(&job_id) {
            if let Some(step) = job.steps.get_mut(step_index) {
                step.status = status;
                step.output = Some(output);
            }
        }
    }

    fn update_job_status(&mut self, job_id: Uuid, status: protocol::Status) {
        let jobs = &mut self.jobs;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = status.clone();
            if matches!(status, protocol::Status::Success | protocol::Status::Failed) {
                jobs.remove(&job_id);
            }
        }
    }
}

impl MessageHandler for Core {
    async fn handle_message(&mut self, message: Message) -> Result<()> {
        debug!("Core received message: {:?}", message);
        match message {
            Message::Api(api_message) => self.handle_api_message(api_message).await?,
            Message::Agent(agent_message) => self.handle_agent_message(agent_message).await?,
            Message::Job(job_message) => self.handle_job_message(job_message).await?,
        }
        Ok(())
    }
}

impl AgentSender for Core {
    async fn send_to_agent(&self, agent_id: Uuid, message: Message) -> Result<()> {
        let agents = &self.agents_manager;

        if let Some(tx) = agents.get_agent_tx(&agent_id) {
            tx.send(message)
                .await
                .with_context(|| format!("Failed to send message to agent {}", agent_id))?;
            Ok(())
        } else {
            Err(anyhow!("Agent with UUID {} not found", agent_id))
        }
    }
}
