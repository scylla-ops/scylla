use async_trait::async_trait;
use protocol::services::orchestrator;
use protocol::services::orchestrator::pipeline_event;
use protocol::uuid::Uuid;
use std::fmt::Debug;

#[async_trait]
pub trait StatusSink: Send + Sync + Debug {
    async fn on_event(&self, event: PipelineEvent);
}

#[derive(Debug, Clone)]
pub enum PipelineEvent {
    Job(JobEvent),
    Stage(StageEvent),
    Step(StepEvent),
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct JobEvent {
    pub id: Uuid,
    pub kind: EventKind,
}

impl From<PipelineEvent> for orchestrator::PipelineEvent {
    fn from(value: PipelineEvent) -> Self {
        match value {
            PipelineEvent::Job(job_e) => orchestrator::PipelineEvent {
                kind: job_e.kind as i32,
                id: job_e.id.to_string(),
                r#type: pipeline_event::EventType::Job as i32,
            },
            PipelineEvent::Stage(stage_e) => orchestrator::PipelineEvent {
                kind: stage_e.kind as i32,
                id: stage_e.id.to_string(),
                r#type: pipeline_event::EventType::Stage as i32,
            },
            PipelineEvent::Step(step_e) => orchestrator::PipelineEvent {
                kind: step_e.kind as i32,
                id: step_e.id.to_string(),
                r#type: pipeline_event::EventType::Step as i32,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageEvent {
    pub id: Uuid,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub struct StepEvent {
    pub id: Uuid,
    pub kind: EventKind,
}
