pub mod pipeline_loader;

pub use serde;
pub use serde::*;
pub use serde_json;
pub use toml;
pub use uuid;

use uuid::Uuid;

/// Statut générique pour job/step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Pending,
    Running,
    Success,
    Failed,
}

/// Définition d'une étape dans une pipeline (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub commands: Vec<String>,
}

/// Définition d'une pipeline (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<PipelineStep>,
}

/// État d'une étape dans un job (exécution d'une pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    pub name: String,
    pub commands: Vec<String>,
    pub status: Status,
    pub output: Option<String>,
}

/// Instance d'exécution d'une pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: Status,
    pub steps: Vec<JobStep>,
}

/// Message principal échangé entre serveur et agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", content = "payload")]
pub enum Message {
    Api(ApiMessage),
    Job(JobMessage),
    Agent(AgentMessage),
}

/// Messages API (ex : exécution de commande)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ApiMessage {
    ExecutePipeline { pipeline: Pipeline },
}

/// Messages relatifs à l'exécution de jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JobMessage {
    /// Demande d'exécution d'un job (instance d'une pipeline)
    Execute { job: Job },
    /// Mise à jour du statut d'une étape (envoyée par l'agent)
    StepStatusUpdate {
        job_id: Uuid,
        step_index: usize,
        status: Status,
        output: String,
    },
    /// Mise à jour globale du job
    JobStatusUpdate { job_id: Uuid, status: Status },
    /// Demande d'annulation d'un job
    Cancel { job_id: Uuid },
}

/// Messages relatifs à l'état des agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentMessage {
    Register { agent_id: Uuid },
    Heartbeat { agent_id: Uuid, status: AgentStatus },
}

/// Statut d'un agent
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Offline,
    Undefined,
}

/// Fonctions utilitaires pour (dé)sérialiser les messages
pub mod utils {
    use super::*;

    pub fn encode_message(msg: &Message) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(msg)
    }

    pub fn decode_message(data: &[u8]) -> Result<Message, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// Trait pour accéder au statut d'un agent
pub trait HasStatus {
    fn status(&self) -> AgentStatus;
}

/// Trait pour accéder à l'UUID d'un agent
pub trait HasUuid {
    fn uuid(&self) -> Uuid;
}
