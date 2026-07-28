//! `Pipeline` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::clock;
use crate::domain::ids::{PipelineId, ProjectId};
use crate::domain::pipeline::{NodeId, PipelineName, Step};
use crate::domain::pipeline::{Pipeline, PipelineNode};
use crate::domain::project::Project;

/// Build a single pipeline node with the given id and deps. Defaults to an
/// `echo <id>` exec step — non-empty by Pipeline rules.
#[must_use]
pub fn node(id: &str, deps: &[&str]) -> PipelineNode {
    PipelineNode::new(
        NodeId::new(id).expect("test node id invalid"),
        deps.iter()
            .map(|d| NodeId::new(*d).expect("test dep id invalid"))
            .collect(),
        Step::exec("echo".into(), vec![id.into()]).expect("test step invalid"),
        None,
        vec![],
    )
}

pub struct PipelineBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl PipelineBuilder {
    /// Default pipeline: a single trivial node `[a]`.
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn)] project: &Project,
        id: Option<PipelineId>,
        #[builder(into, default = "test-pipeline".to_string())] name: String,
        nodes: Option<Vec<PipelineNode>>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Pipeline {
        Self::assemble_from_project_id(
            project.id().clone(),
            id,
            name,
            nodes,
            created_at,
            updated_at,
        )
    }

    #[builder(start_fn = for_project_id, finish_fn = build)]
    pub fn assemble_from_project_id(
        #[builder(start_fn)] project_id: ProjectId,
        id: Option<PipelineId>,
        #[builder(into, default = "test-pipeline".to_string())] name: String,
        nodes: Option<Vec<PipelineNode>>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Pipeline {
        let now = created_at.unwrap_or_else(clock::now);
        let nodes = nodes.unwrap_or_else(|| vec![node("a", &[])]);
        Pipeline::from_persistence(
            id.unwrap_or_else(PipelineId::generate),
            project_id,
            PipelineName::new(name).expect("test pipeline name invalid"),
            nodes,
            now,
            updated_at.unwrap_or(now),
        )
    }
}

#[must_use]
pub fn pipeline(project: &Project) -> Pipeline {
    PipelineBuilder::new(project).build()
}

pub async fn seed_pipeline(pool: &sqlx::PgPool, project: &Project) -> Pipeline {
    use crate::application::PipelineRepository;
    use crate::infrastructure::persistence::postgres::PgPipelineRepository;
    let pipeline = pipeline(project);
    PgPipelineRepository::new(pool.clone())
        .create(&pipeline)
        .await
        .expect("seed pipeline failed");
    pipeline
}
