//! The adapter: `create_trigger` and `list_pipeline_triggers` are one `run` and their response.
//! The other RPCs call the use case directly: their permission is on the loaded trigger's
//! pipeline, which `Describe` cannot see.

use crate::application::{TriggerFireUseCases, TriggerUseCases};
use crate::extract_auth_context;
use crate::grpc::adapter::run;
use crate::grpc::convert::{id, valid, wrap};
use crate::grpc::mappers::domain_error_to_status;
use crate::grpc::mappers::trigger_mapper::{
    proto_inputs_to_domain, trigger_to_proto, update_source_to_domain,
};
use derive_more::Constructor;
use scylla_domain::domain::ids::TriggerId;
use scylla_domain::domain::trigger::{Trigger, TriggerName};
use scylla_extension::Actions;
use scylla_proto::trigger::v1::{
    CreateTriggerRequest, CreateTriggerResponse, DeleteTriggerRequest, DeleteTriggerResponse,
    FireTriggerNowRequest, FireTriggerNowResponse, GetTriggerRequest, GetTriggerResponse,
    ListPipelineTriggersRequest, ListPipelineTriggersResponse, SetTriggerEnabledRequest,
    SetTriggerEnabledResponse, Trigger as ProtoTrigger, UpdateTriggerRequest,
    UpdateTriggerResponse, trigger_service_server::TriggerService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct TriggerHandler {
    actions: Arc<Actions>,
    triggers: Arc<TriggerUseCases>,
    fire_uc: Arc<TriggerFireUseCases>,
    webhook_base_url: Option<String>,
}

impl TriggerHandler {
    fn view(&self, trigger: &Trigger) -> ProtoTrigger {
        trigger_to_proto(trigger, self.webhook_base_url.as_deref())
    }
}

#[async_trait::async_trait]
impl TriggerService for TriggerHandler {
    async fn create_trigger(
        &self,
        request: Request<CreateTriggerRequest>,
    ) -> Result<Response<CreateTriggerResponse>, Status> {
        let (trigger, webhook_secret) = run(&self.actions, &*self.triggers, request).await?;
        Ok(Response::new(CreateTriggerResponse {
            trigger: Some(self.view(&trigger)),
            webhook_secret,
        }))
    }

    async fn get_trigger(
        &self,
        request: Request<GetTriggerRequest>,
    ) -> Result<Response<GetTriggerResponse>, Status> {
        let caller = caller!(request);
        let id: TriggerId = id(request.into_inner().trigger_id, "trigger_id")?;
        let trigger = self
            .triggers
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(GetTriggerResponse {
            trigger: Some(self.view(&trigger)),
        }))
    }

    async fn update_trigger(
        &self,
        request: Request<UpdateTriggerRequest>,
    ) -> Result<Response<UpdateTriggerResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id: TriggerId = id(req.trigger_id, "trigger_id")?;
        let name = valid(
            req.name
                .ok_or_else(|| Status::invalid_argument("missing name"))?,
            TriggerName::new,
        )?;
        let source = update_source_to_domain(req.source)?;
        let inputs = proto_inputs_to_domain(req.inputs)?;
        let trigger = self
            .triggers
            .update(&caller, &id, name, source, inputs)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(UpdateTriggerResponse {
            trigger: Some(self.view(&trigger)),
        }))
    }

    async fn delete_trigger(
        &self,
        request: Request<DeleteTriggerRequest>,
    ) -> Result<Response<DeleteTriggerResponse>, Status> {
        let caller = caller!(request);
        let id: TriggerId = id(request.into_inner().trigger_id, "trigger_id")?;
        self.triggers
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteTriggerResponse {}))
    }

    async fn list_pipeline_triggers(
        &self,
        request: Request<ListPipelineTriggersRequest>,
    ) -> Result<Response<ListPipelineTriggersResponse>, Status> {
        let triggers = run(&self.actions, &*self.triggers, request).await?;
        Ok(Response::new(ListPipelineTriggersResponse {
            triggers: triggers.iter().map(|t| self.view(t)).collect(),
        }))
    }

    async fn set_trigger_enabled(
        &self,
        request: Request<SetTriggerEnabledRequest>,
    ) -> Result<Response<SetTriggerEnabledResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id: TriggerId = id(req.trigger_id, "trigger_id")?;
        let trigger = self
            .triggers
            .set_enabled(&caller, &id, req.enabled)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(SetTriggerEnabledResponse {
            trigger: Some(self.view(&trigger)),
        }))
    }

    async fn fire_trigger_now(
        &self,
        request: Request<FireTriggerNowRequest>,
    ) -> Result<Response<FireTriggerNowResponse>, Status> {
        let caller = caller!(request);
        let id: TriggerId = id(request.into_inner().trigger_id, "trigger_id")?;
        let job = self
            .fire_uc
            .fire_now(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(FireTriggerNowResponse {
            job_id: wrap(job.id().to_string()),
        }))
    }
}
