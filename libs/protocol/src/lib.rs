pub mod pipeline_loader;

pub use serde;
pub use serde::*;
pub use serde_json;
pub use toml;
pub use uuid;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Generic status for job/step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Pending,
    Running,
    Success,
    Failed,
}

/// Definition of a step in a pipeline (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub commands: Vec<String>,
}

/// Definition of a pipeline (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<PipelineStep>,
}

/// State of a step in a job (execution of a pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    pub name: String,
    pub commands: Vec<String>,
    pub status: Status,
    pub output: Option<String>,
}

/// Execution instance of a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: Status,
    pub steps: Vec<JobStep>,
}

/// Main message exchanged between server and agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", content = "payload")]
pub enum Message {
    Api(ApiMessage),
    Job(JobMessage),
    Agent(AgentMessage),
}

/// API messages (e.g.: command execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ApiMessage {
    ExecutePipeline { pipeline: Pipeline },
}

/// Messages related to job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobMessage {
    /// Request to execute a job (instance of a pipeline)
    Execute { job: Job },
    /// Update of a step status (sent by the agent)
    StepStatusUpdate {
        job_id: Uuid,
        step_index: usize,
        status: Status,
        output: String,
    },
    /// Global update of the job
    JobStatusUpdate { job_id: Uuid, status: Status },
    /// Request to cancel a job
    Cancel { job_id: Uuid },
}

/// Messages related to the agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentMessage {
    Register {
        agent_id: Uuid,
    },
    Heartbeat {
        agent_id: Uuid,
        status: AgentStatus,
    },

    // Internal messages used by the server to manage agent connections
    #[serde(skip_serializing, skip_deserializing)]
    Connected {
        agent_id: Uuid,
        tx: tokio::sync::mpsc::Sender<Message>,
    },

    #[serde(skip_serializing, skip_deserializing)]
    Disconnected {
        agent_id: Uuid,
    },
}

/// Status of an agent
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Offline,
    Undefined,
}

/// Utility functions to (de)serialize messages
pub mod utils {
    use super::*;

    pub fn encode_message(msg: &Message) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(msg)
    }

    pub fn decode_message(data: &[u8]) -> Result<Message, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// Trait to access an agent's status
pub trait HasStatus {
    fn status(&self) -> AgentStatus;
}

/// Trait to access an agent's UUID
pub trait HasUuid {
    fn uuid(&self) -> Uuid;
}

/// Trait for handling different types of messages
#[async_trait]
pub trait MessageHandler {
    /// Handle a message and return a result
    async fn handle_message(&mut self, message: Message) -> Result<()>;
}
