use crate::application::{
    AgentDispatch, AppRepository, HashService, JobRepository, PermissionService,
    PipelineRepository, PolicyControl, ProjectRepository, TriggerFireUseCases, TriggerRepository,
    TriggerUseCases,
};
use crate::extract_auth_context;
use crate::grpc::convert::{required, ts, wrap};
use crate::grpc::mappers::domain_error_to_status;
use scylla_core::domain::ids::{PipelineId, TriggerId};
use scylla_core::domain::pipeline::EnvKey;
use scylla_core::domain::trigger::{
    CronSpec, TriggerInput, TriggerInputSource, TriggerName, TriggerSource, WebhookSpec,
};
use scylla_core::domain::trigger::{FireObservation, Trigger, TriggerActivation};
use scylla_protocol::trigger::v1::{
    CreateTriggerRequest, CreateTriggerResponse, CronSpec as ProtoCronSpec, DeleteTriggerRequest,
    DeleteTriggerResponse, FireObservation as ProtoFireObservation, FireTriggerNowRequest,
    FireTriggerNowResponse, GetTriggerRequest, GetTriggerResponse, ListPipelineTriggersRequest,
    ListPipelineTriggersResponse, SetTriggerEnabledRequest, SetTriggerEnabledResponse,
    Trigger as ProtoTrigger, TriggerInput as ProtoTriggerInput, UpdateTriggerRequest,
    UpdateTriggerResponse, WebhookSpec as ProtoWebhookSpec, create_trigger_request,
    fire_observation, trigger as proto_trigger, trigger_input,
    trigger_service_server::TriggerService, update_trigger_request,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// gRPC surface for managing a pipeline's triggers. CRUD is delegated to
/// [`TriggerUseCases`] (Cedar-gated by `manageTriggers`, with the create/update
/// anti-escalation `runPipeline` check); manual firing is authorized against the
/// caller's `runPipeline` on the trigger's pipeline, then goes through
/// [`TriggerFireUseCases`], which runs as the org's trigger-runner App. On create,
/// a webhook trigger's generated signing secret is returned ONCE in
/// `webhook_secret`; the delivery URL lives inside the read model's webhook arm
/// and is built from the configured ingress base URL.
pub struct TriggerHandler<T, P, PR, A, H, PC, PS, J, W>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
    J: JobRepository,
    W: AgentDispatch,
{
    use_cases: Arc<TriggerUseCases<T, P, PR, A, H, PC, PS>>,
    fire_uc: Arc<TriggerFireUseCases<T, P, PR, A, J, PS, W>>,
    /// Public base URL of the webhook ingress (e.g. `https://host:8088`), used to
    /// build `Trigger.webhook.url`. `None` when ingress isn't configured.
    webhook_base_url: Option<String>,
}

impl<T, P, PR, A, H, PC, PS, J, W> TriggerHandler<T, P, PR, A, H, PC, PS, J, W>
where
    T: TriggerRepository,
    P: PipelineRepository,
    PR: ProjectRepository,
    A: AppRepository,
    H: HashService,
    PC: PolicyControl,
    PS: PermissionService,
    J: JobRepository,
    W: AgentDispatch,
{
    pub fn new(
        use_cases: Arc<TriggerUseCases<T, P, PR, A, H, PC, PS>>,
        fire_uc: Arc<TriggerFireUseCases<T, P, PR, A, J, PS, W>>,
        webhook_base_url: Option<String>,
    ) -> Self {
        Self {
            use_cases,
            fire_uc,
            webhook_base_url,
        }
    }

    /// Map a domain trigger to its proto read model, filling the webhook arm's
    /// `url` from the configured ingress base URL.
    fn view(&self, trigger: &Trigger) -> ProtoTrigger {
        trigger_to_view(trigger, self.webhook_base_url.as_deref())
    }
}

#[async_trait::async_trait]
impl<T, P, PR, A, H, PC, PS, J, W> TriggerService for TriggerHandler<T, P, PR, A, H, PC, PS, J, W>
where
    T: TriggerRepository + Send + Sync + 'static,
    P: PipelineRepository + Send + Sync + 'static,
    PR: ProjectRepository + Send + Sync + 'static,
    A: AppRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    PC: PolicyControl + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
    J: JobRepository + Send + Sync + 'static,
    W: AgentDispatch + Send + Sync + 'static,
{
    async fn create_trigger(
        &self,
        request: Request<CreateTriggerRequest>,
    ) -> Result<Response<CreateTriggerResponse>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();

        let pipeline_id = PipelineId::new(&required(req.pipeline_id, "pipeline_id")?);
        let name = TriggerName::new(&req.name).map_err(domain_error_to_status)?;
        let source = create_source_to_domain(req.source)?;
        let inputs = proto_inputs_to_domain(req.inputs)?;

        let (trigger, webhook_secret) = self
            .use_cases
            .create(&caller, pipeline_id, name, source, inputs)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateTriggerResponse {
            trigger: Some(self.view(&trigger)),
            // Returned ONCE for webhook triggers; absent for cron.
            webhook_secret,
        }))
    }

    async fn get_trigger(
        &self,
        request: Request<GetTriggerRequest>,
    ) -> Result<Response<GetTriggerResponse>, Status> {
        let caller = caller!(request);
        let id = TriggerId::new(&required(request.into_inner().trigger_id, "trigger_id")?);

        let trigger = self
            .use_cases
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

        let id = TriggerId::new(&required(req.trigger_id, "trigger_id")?);
        let name = TriggerName::new(
            req.name
                .ok_or_else(|| Status::invalid_argument("missing name"))?,
        )
        .map_err(domain_error_to_status)?;
        let source = update_source_to_domain(req.source)?;
        let inputs = proto_inputs_to_domain(req.inputs)?;

        let trigger = self
            .use_cases
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
        let id = TriggerId::new(&required(request.into_inner().trigger_id, "trigger_id")?);

        self.use_cases
            .delete(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteTriggerResponse {}))
    }

    async fn list_pipeline_triggers(
        &self,
        request: Request<ListPipelineTriggersRequest>,
    ) -> Result<Response<ListPipelineTriggersResponse>, Status> {
        let caller = caller!(request);
        let pipeline_id =
            PipelineId::new(&required(request.into_inner().pipeline_id, "pipeline_id")?);

        let triggers = self
            .use_cases
            .list_by_pipeline(&caller, &pipeline_id)
            .await
            .map_err(domain_error_to_status)?;

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
        let id = TriggerId::new(&required(req.trigger_id, "trigger_id")?);

        let trigger = self
            .use_cases
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
        let id = TriggerId::new(&required(request.into_inner().trigger_id, "trigger_id")?);

        // Manual fire is authorized inside the use case (`RunPipeline` on the
        // trigger's pipeline), then runs as the org's trigger-runner App.
        let job = self
            .fire_uc
            .fire_now(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        // Only the minted job's id: fetch the job itself with JobService.GetJob.
        Ok(Response::new(FireTriggerNowResponse {
            job_id: wrap(job.id().to_string()),
        }))
    }
}

// ── proto → domain ───────────────────────────────────────────────────────────

fn create_source_to_domain(
    source: Option<create_trigger_request::Source>,
) -> Result<TriggerSource, Status> {
    match source {
        Some(create_trigger_request::Source::Cron(c)) => cron_to_domain(c),
        Some(create_trigger_request::Source::Webhook(w)) => webhook_to_domain(w),
        None => Err(Status::invalid_argument(
            "trigger source is required (cron or webhook)",
        )),
    }
}

fn update_source_to_domain(
    source: Option<update_trigger_request::Source>,
) -> Result<TriggerSource, Status> {
    match source {
        Some(update_trigger_request::Source::Cron(c)) => cron_to_domain(c),
        Some(update_trigger_request::Source::Webhook(w)) => webhook_to_domain(w),
        None => Err(Status::invalid_argument(
            "trigger source is required (cron or webhook)",
        )),
    }
}

fn cron_to_domain(c: ProtoCronSpec) -> Result<TriggerSource, Status> {
    Ok(TriggerSource::Cron(
        CronSpec::new(c.expression).map_err(domain_error_to_status)?,
    ))
}

fn webhook_to_domain(w: ProtoWebhookSpec) -> Result<TriggerSource, Status> {
    let header = (!w.signature_header.trim().is_empty()).then_some(w.signature_header);
    Ok(TriggerSource::Webhook(
        WebhookSpec::new(header).map_err(domain_error_to_status)?,
    ))
}

fn proto_inputs_to_domain(inputs: Vec<ProtoTriggerInput>) -> Result<Vec<TriggerInput>, Status> {
    inputs
        .into_iter()
        .map(|input| {
            let key = EnvKey::new(&input.key).map_err(domain_error_to_status)?;
            match input.source {
                Some(trigger_input::Source::Literal(value)) => {
                    Ok(TriggerInput::literal(key, value))
                }
                Some(trigger_input::Source::JsonPointer(pointer)) => {
                    TriggerInput::json_pointer(key, pointer).map_err(domain_error_to_status)
                }
                None => Err(Status::invalid_argument(format!(
                    "trigger input '{}' has no source (literal or json_pointer)",
                    input.key
                ))),
            }
        })
        .collect()
}

// ── domain → proto ───────────────────────────────────────────────────────────

fn trigger_to_view(t: &Trigger, webhook_base_url: Option<&str>) -> ProtoTrigger {
    ProtoTrigger {
        trigger_id: wrap(t.id().to_string()),
        pipeline_id: wrap(t.pipeline_id().to_string()),
        name: t.name().to_string(),
        source: Some(source_to_proto(t, webhook_base_url)),
        inputs: t.inputs().iter().map(input_to_proto).collect(),
        activation: Some(activation_to_proto(t.activation())),
        last_observation: t.last_observation().map(observation_to_proto),
        created_at: ts(t.created_at()),
        updated_at: ts(t.updated_at()),
    }
}

fn activation_to_proto(activation: &TriggerActivation) -> proto_trigger::Activation {
    match activation {
        TriggerActivation::Disabled => {
            proto_trigger::Activation::Disabled(proto_trigger::Disabled {})
        }
        TriggerActivation::Enabled { next_fire_at } => {
            proto_trigger::Activation::Enabled(proto_trigger::Enabled {
                next_fire_at: next_fire_at.and_then(ts),
            })
        }
    }
}

/// The domain records `"ok"` on success and an error description otherwise.
fn observation_to_proto(observation: &FireObservation) -> ProtoFireObservation {
    let result = if observation.status == "ok" {
        fire_observation::Result::Succeeded(fire_observation::Succeeded {})
    } else {
        fire_observation::Result::Failed(fire_observation::Failed {
            error: observation.status.clone(),
        })
    };
    ProtoFireObservation {
        fired_at: ts(observation.fired_at),
        result: Some(result),
    }
}

/// Read-side source union. The delivery URL lives in the webhook arm and is
/// built from the configured ingress base; an unconfigured ingress leaves it
/// empty. A cron trigger has no URL field at all.
fn source_to_proto(t: &Trigger, webhook_base_url: Option<&str>) -> proto_trigger::Source {
    match t.source() {
        TriggerSource::Cron(c) => proto_trigger::Source::Cron(proto_trigger::Cron {
            expression: c.expression().to_string(),
        }),
        TriggerSource::Webhook(w) => proto_trigger::Source::Webhook(proto_trigger::Webhook {
            signature_header: w.signature_header().unwrap_or_default().to_string(),
            url: webhook_base_url
                .map(|base| format!("{}/webhooks/{}", base.trim_end_matches('/'), t.id()))
                .unwrap_or_default(),
        }),
    }
}

fn input_to_proto(input: &TriggerInput) -> ProtoTriggerInput {
    let source = match input.source() {
        TriggerInputSource::Literal(value) => trigger_input::Source::Literal(value.clone()),
        TriggerInputSource::JsonPointer(pointer) => {
            trigger_input::Source::JsonPointer(pointer.clone())
        }
    };
    ProtoTriggerInput {
        key: input.key().to_string(),
        source: Some(source),
    }
}
