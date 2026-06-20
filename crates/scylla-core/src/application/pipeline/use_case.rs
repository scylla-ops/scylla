use crate::application::agent::dispatch::{DispatchEnv, DispatchNode};
use crate::application::caller::CallerContext;
use crate::application::{
    JobDispatch, JobRepository, PermissionService, PipelineRepository, ProjectRepository,
    SecretResolver,
};
use crate::domain::entities::{
    AppId, Job, JobId, OrganizationId, Pipeline, PipelineId, PipelineNode, ProjectId,
};
use crate::domain::errors::DomainResult;
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::pipeline::PipelineName;
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct PipelineUseCases<
    P: PipelineRepository,
    PR: ProjectRepository,
    J: JobRepository,
    PS: PermissionService,
> {
    pipeline_repo: Arc<P>,
    project_repo: Arc<PR>,
    job_repo: Arc<J>,
    permission_service: Arc<PS>,
    secret_resolver: Arc<dyn SecretResolver>,
}

impl<P: PipelineRepository, PR: ProjectRepository, J: JobRepository, PS: PermissionService>
    PipelineUseCases<P, PR, J, PS>
{
    #[instrument(skip(self, caller, nodes), fields(name = %name, project_id = %project_id))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: PipelineName,
        project_id: ProjectId,
        nodes: Vec<PipelineNode>,
    ) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::CreatePipeline(project_id.clone()))
            .await?;
        self.project_repo.find_by_id(&project_id).await?;
        let pipeline = Pipeline::create(name, project_id, nodes)?;
        self.pipeline_repo.create(&pipeline).await
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %id))]
    pub async fn get(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::ReadPipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await
    }

    // Note: previously had `find_internal` for orchestration paths to bypass the
    // get-check; replaced by the consolidated `run()` method below which does a
    // single `Permission::RunPipeline` check and reads the pipeline via the repo.

    #[instrument(skip(self, caller, nodes), fields(pipeline_id = %id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &PipelineId,
        name: Option<PipelineName>,
        nodes: Option<Vec<PipelineNode>>,
    ) -> DomainResult<Pipeline> {
        self.permission_service
            .check(caller, Permission::UpdatePipeline(id.clone()))
            .await?;

        let mut pipeline = self.pipeline_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            pipeline.update_name(new_name)?;
        }
        if let Some(new_nodes) = nodes {
            pipeline.update_nodes(new_nodes)?;
        }

        self.pipeline_repo.update(&pipeline).await
    }

    #[instrument(skip(self, caller), fields(pipeline_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &PipelineId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeletePipeline(id.clone()))
            .await?;
        self.pipeline_repo.find_by_id(id).await?;
        self.pipeline_repo.delete(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(caller, Permission::ListPipelines)
            .await?;
        self.pipeline_repo.list_all(pagination).await
    }

    #[instrument(skip(self, caller), fields(project_id = %project_id))]
    pub async fn list_by_project(
        &self,
        caller: &CallerContext,
        project_id: &ProjectId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(
                caller,
                Permission::ListPipelinesByProject(project_id.clone()),
            )
            .await?;
        self.pipeline_repo
            .list_by_project(project_id, pagination)
            .await
    }

    #[instrument(skip(self, caller), fields(org_id = %organization_id))]
    pub async fn list_by_organization(
        &self,
        caller: &CallerContext,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        self.permission_service
            .check(
                caller,
                Permission::ListPipelinesByOrganization(organization_id.clone()),
            )
            .await?;
        self.pipeline_repo
            .list_by_organization(organization_id, pagination)
            .await
    }

    /// Authorize + materialise the run of `pipeline_id` for `caller`. Loads the
    /// pipeline, mints a `Job`, persists it, and returns the dispatch payload.
    /// **Single** permission check (`Permission::RunPipeline`) — the internal
    /// repo calls deliberately bypass per-step Cedar so granting "run" doesn't
    /// also require "get" and "create-job". The caller (handler / firing engine)
    /// then hands the payload to an agent via `DispatchUseCases` (in-process; no
    /// broker).
    pub async fn run(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
    ) -> DomainResult<(Job, JobDispatch)> {
        self.run_with_inputs(caller, pipeline_id, &[]).await
    }

    /// Like [`run`](Self::run) but overlays `inputs` — already-resolved
    /// `(key, value)` env pairs, e.g. from a trigger — onto every node as
    /// literal (unmasked) env, merged AFTER secret resolution. A node's own env
    /// wins on a key collision, and inputs are plain literals that can never
    /// reference a secret. Same single `RunPipeline` check as `run`.
    #[instrument(skip(self, caller, inputs), fields(pipeline_id = %pipeline_id, inputs = inputs.len()))]
    pub async fn run_with_inputs(
        &self,
        caller: &CallerContext,
        pipeline_id: &PipelineId,
        inputs: &[(String, String)],
    ) -> DomainResult<(Job, JobDispatch)> {
        self.permission_service
            .check(caller, Permission::RunPipeline(pipeline_id.clone()))
            .await?;

        let pipeline = self.pipeline_repo.find_by_id(pipeline_id).await?;
        let job = Job::create_from_pipeline(&pipeline);
        let job = self.job_repo.create(&job).await?;
        // Resolve secret-ref env vars (decrypt), then overlay trigger inputs.
        let nodes = self
            .secret_resolver
            .resolve(pipeline.project_id(), pipeline.nodes())
            .await?;
        let nodes = apply_inputs(nodes, inputs);
        let dispatch = JobDispatch {
            job_id: job.id().to_string(),
            pipeline_id: pipeline.id().to_string(),
            nodes,
        };
        Ok((job, dispatch))
    }

    /// Record which agent the job was dispatched to. An internal continuation
    /// of the already-authorized `run` (the handler calls this once an agent
    /// accepts the dispatch), so it carries no extra Cedar check.
    #[instrument(skip(self), fields(job_id = %job_id, app_id = %app_id))]
    pub async fn assign_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        self.job_repo.set_agent(job_id, app_id).await
    }
}

/// Overlay trigger-supplied `inputs` onto each dispatch node as literal
/// (unmasked) env. Applied after secret resolution; a node's own env wins on a
/// key collision, so a trigger can add context (e.g. `GIT_COMMIT`) but never
/// override or shadow what the pipeline defined.
fn apply_inputs(mut nodes: Vec<DispatchNode>, inputs: &[(String, String)]) -> Vec<DispatchNode> {
    if inputs.is_empty() {
        return nodes;
    }
    for node in &mut nodes {
        let existing: HashSet<String> = node.env.iter().map(|e| e.key.clone()).collect();
        for (key, value) in inputs {
            if !existing.contains(key) {
                node.env.push(DispatchEnv {
                    key: key.clone(),
                    value: value.clone(),
                    masked: false,
                });
            }
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::pipeline::Step;

    fn node(env: &[(&str, &str)]) -> DispatchNode {
        DispatchNode {
            id: "n".to_string(),
            deps: vec![],
            working_dir: None,
            step: Step::exec("echo".to_string(), vec![]).unwrap(),
            env: env
                .iter()
                .map(|(k, v)| DispatchEnv {
                    key: (*k).to_string(),
                    value: (*v).to_string(),
                    masked: false,
                })
                .collect(),
        }
    }

    fn env_of(n: &DispatchNode, key: &str) -> Option<String> {
        n.env.iter().find(|e| e.key == key).map(|e| e.value.clone())
    }

    #[test]
    fn empty_inputs_is_noop() {
        let nodes = apply_inputs(vec![node(&[("A", "1")])], &[]);
        assert_eq!(nodes[0].env.len(), 1);
    }

    #[test]
    fn inputs_added_as_unmasked_env() {
        let nodes = apply_inputs(
            vec![node(&[])],
            &[("GIT_COMMIT".to_string(), "abc".to_string())],
        );
        let e = nodes[0].env.iter().find(|e| e.key == "GIT_COMMIT").unwrap();
        assert_eq!(e.value, "abc");
        assert!(!e.masked, "trigger inputs are unmasked literals");
    }

    #[test]
    fn node_env_wins_on_collision() {
        let nodes = apply_inputs(
            vec![node(&[("MODE", "node")])],
            &[("MODE".to_string(), "trigger".to_string())],
        );
        assert_eq!(env_of(&nodes[0], "MODE").as_deref(), Some("node"));
        assert_eq!(nodes[0].env.iter().filter(|e| e.key == "MODE").count(), 1);
    }

    #[test]
    fn inputs_applied_to_all_nodes() {
        let nodes = apply_inputs(
            vec![node(&[]), node(&[("X", "1")])],
            &[("RUN_MODE".to_string(), "nightly".to_string())],
        );
        assert_eq!(env_of(&nodes[0], "RUN_MODE").as_deref(), Some("nightly"));
        assert_eq!(env_of(&nodes[1], "RUN_MODE").as_deref(), Some("nightly"));
    }
}
