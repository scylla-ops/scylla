//! The trigger's reads. One block per query, in the order it runs: the struct, its permission,
//! its output type, what `Fetch` reads.

use super::TriggerUseCases;
use crate::application::{
    AppRepository, HashService, PipelineRepository, ProjectRepository, TriggerRepository,
};
use crate::domain::errors::DomainResult;
use crate::domain::ids::PipelineId;
use crate::domain::permission::Permission;
use crate::domain::trigger::Trigger;
use async_trait::async_trait;
use scylla_auth::authz::{PermissionService, PolicyControl};
use scylla_extension::{Authorized, Describe, Fetch, Fetched, Query, Run};

#[derive(Debug)]
pub struct ListPipelineTriggers {
    pub pipeline_id: PipelineId,
}

impl Describe for ListPipelineTriggers {
    fn permission(&self) -> Permission {
        Permission::ManageTriggers(self.pipeline_id.clone())
    }
}

impl Query for ListPipelineTriggers {
    type Output = Vec<Trigger>;
}

#[async_trait]
impl<T, P, PR, A, H, PC, PS> Run<Fetch<ListPipelineTriggers>>
    for TriggerUseCases<T, P, PR, A, H, PC, PS>
where
    T: TriggerRepository + Send + Sync,
    P: PipelineRepository + Send + Sync,
    PR: ProjectRepository + Send + Sync,
    A: AppRepository,
    H: HashService + Send + Sync,
    PC: PolicyControl,
    PS: PermissionService,
{
    async fn run(
        &self,
        input: Authorized<ListPipelineTriggers>,
    ) -> DomainResult<Fetched<ListPipelineTriggers>> {
        let triggers = self
            .trigger_repo
            .list_by_pipeline(&input.command().pipeline_id)
            .await?;
        Ok(input.fetched(triggers))
    }
}
