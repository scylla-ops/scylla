use crate::extract_auth_context;
use crate::grpc::convert::{required, ts, wrap};
use crate::grpc::mappers::{domain_error_to_status, job_to_proto};
use scylla_core::application::{
    AgentDispatch, AppRepository, HashService, JobRepository, PermissionService, PipelineRepository,
    PolicyControl, ProjectRepository, TriggerFireUseCases, TriggerRepository, TriggerUseCases,
};
use scylla_core::domain::entities::{PipelineId, Trigger, TriggerId};
use scylla_core::domain::value_objects::pipeline::EnvKey;
use scylla_core::domain::value_objects::trigger::{
    CronSpec, TriggerInput, TriggerInputSource, TriggerName, TriggerSource, WebhookSpec,
};
use scylla_protocol::services::job::JobResponse;
use scylla_protocol::services::trigger::{
    CreateTriggerRequest, CreatedTrigger, CronSpec as ProtoCronSpec, DeleteTriggerRequest,
    DeleteTriggerResponse, FireTriggerNowRequest, GetTriggerRequest, ListPipelineTriggersRequest,
    ListTriggersResponse, SetTriggerEnabledRequest, TriggerInput as ProtoTriggerInput, TriggerView,
    UpdateTriggerRequest, WebhookSpec as ProtoWebhookSpec, create_trigger_request,
    trigger_input, trigger_service_server::TriggerService, trigger_view, update_trigger_request,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// gRPC surface for managing a pipeline's triggers. CRUD is delegated to
/// [`TriggerUseCases`] (Cedar-gated by `manageTriggers`, with the create/update
/// anti-escalation `runPipeline` check); manual firing is authorized against the
/// caller's `runPipeline` on the trigger's pipeline, then goes through
/// [`TriggerFireUseCases`], which runs as the org's trigger-runner App. On create,
/// a webhook trigger's generated signing secret is returned ONCE in
/// `webhook_secret`; `webhook_url` in every view is built from the configured
/// ingress base URL.
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
    /// build `TriggerView.webhook_url`. `None` when ingress isn't configured.
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

    /// Map a domain trigger to its proto view, filling `webhook_url` from the
    /// configured ingress base URL.
    fn view(&self, trigger: &Trigger) -> TriggerView {
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
    ) -> Result<Response<CreatedTrigger>, Status> {
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

        Ok(Response::new(CreatedTrigger {
            trigger: Some(self.view(&trigger)),
            // Returned ONCE for webhook triggers; empty for cron.
            webhook_secret: webhook_secret.unwrap_or_default(),
        }))
    }

    async fn get_trigger(
        &self,
        request: Request<GetTriggerRequest>,
    ) -> Result<Response<TriggerView>, Status> {
        let caller = caller!(request);
        let id = TriggerId::new(&required(request.into_inner().trigger_id, "trigger_id")?);

        let trigger = self
            .use_cases
            .get(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(self.view(&trigger)))
    }

    async fn update_trigger(
        &self,
        request: Request<UpdateTriggerRequest>,
    ) -> Result<Response<TriggerView>, Status> {
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

        Ok(Response::new(self.view(&trigger)))
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
    ) -> Result<Response<ListTriggersResponse>, Status> {
        let caller = caller!(request);
        let pipeline_id =
            PipelineId::new(&required(request.into_inner().pipeline_id, "pipeline_id")?);

        let triggers = self
            .use_cases
            .list_by_pipeline(&caller, &pipeline_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListTriggersResponse {
            triggers: triggers.iter().map(|t| self.view(t)).collect(),
        }))
    }

    async fn set_trigger_enabled(
        &self,
        request: Request<SetTriggerEnabledRequest>,
    ) -> Result<Response<TriggerView>, Status> {
        let caller = caller!(request);
        let req = request.into_inner();
        let id = TriggerId::new(&required(req.trigger_id, "trigger_id")?);

        let trigger = self
            .use_cases
            .set_enabled(&caller, &id, req.enabled)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(self.view(&trigger)))
    }

    async fn fire_trigger_now(
        &self,
        request: Request<FireTriggerNowRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let caller = caller!(request);
        let id = TriggerId::new(&required(request.into_inner().trigger_id, "trigger_id")?);

        // Manual fire is authorized inside the use case (`RunPipeline` on the
        // trigger's pipeline), then runs as the org's trigger-runner App.
        let job = self
            .fire_uc
            .fire_now(&caller, &id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(job_to_proto(&job)))
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
                Some(trigger_input::Source::Literal(value)) => Ok(TriggerInput::literal(key, value)),
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

fn trigger_to_view(t: &Trigger, webhook_base_url: Option<&str>) -> TriggerView {
    // Webhook triggers expose a delivery URL built from the configured ingress
    // base; cron triggers (and an unconfigured ingress) leave it empty.
    let webhook_url = match (t.source(), webhook_base_url) {
        (TriggerSource::Webhook(_), Some(base)) => {
            format!("{}/webhooks/{}", base.trim_end_matches('/'), t.id())
        }
        _ => String::new(),
    };
    TriggerView {
        trigger_id: wrap(t.id().to_string()),
        pipeline_id: wrap(t.pipeline_id().to_string()),
        name: t.name().to_string(),
        source: Some(source_to_proto(t.source())),
        inputs: t.inputs().iter().map(input_to_proto).collect(),
        enabled: t.is_enabled(),
        webhook_url,
        next_fire_at: t.next_fire_at().and_then(ts),
        last_fired_at: t.last_fired_at().and_then(ts),
        last_status: t.last_status().unwrap_or_default().to_string(),
        created_at: ts(t.created_at()),
        updated_at: ts(t.updated_at()),
    }
}

fn source_to_proto(source: &TriggerSource) -> trigger_view::Source {
    match source {
        TriggerSource::Cron(c) => trigger_view::Source::Cron(ProtoCronSpec {
            expression: c.expression().to_string(),
        }),
        TriggerSource::Webhook(w) => trigger_view::Source::Webhook(ProtoWebhookSpec {
            signature_header: w.signature_header().unwrap_or_default().to_string(),
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
