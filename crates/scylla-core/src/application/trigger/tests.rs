//! The trigger's actions through the engine, on stub ports.

use super::*;
use crate::application::{DispatchUseCases, PipelineUseCases};
use crate::domain::agent::Agent;
use crate::domain::caller::{CallerContext, ServiceIdentity};
use crate::domain::ids::{AppId, PipelineId, ProjectId, TriggerId};
use crate::domain::job::Job;
use crate::domain::permission::Permission;
use crate::domain::trigger::{CronSpec, TriggerName, TriggerSource, WebhookSpec};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::pipelines::PipelineBuilder;
use crate::test_support::projects::ProjectBuilder;
use crate::test_support::stubs::{
    CountingPolicy, EchoResolver, OnePipeline, OneProject, StubHash, StubJobs, StubRegistry, alice,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_auth::authz::PermissionService;
use scylla_extension::{Actions, Deleted};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubTriggers {
    rows: Mutex<HashMap<TriggerId, (Trigger, Option<Vec<u8>>)>>,
}

#[async_trait]
impl TriggerRepository for StubTriggers {
    async fn create(&self, trigger: &Trigger, enc: Option<&[u8]>) -> DomainResult<Trigger> {
        self.rows.lock().unwrap().insert(
            trigger.id().clone(),
            (trigger.clone(), enc.map(<[u8]>::to_vec)),
        );
        Ok(trigger.clone())
    }
    async fn find_by_id(&self, id: &TriggerId) -> DomainResult<Trigger> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .map(|(t, _)| t.clone())
            .ok_or_else(|| DomainError::not_found("Trigger", id.to_string()))
    }
    async fn webhook_secret(&self, id: &TriggerId) -> DomainResult<Option<Vec<u8>>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .get(id)
            .and_then(|(_, enc)| enc.clone()))
    }
    async fn update(&self, trigger: &Trigger) -> DomainResult<Trigger> {
        let mut rows = self.rows.lock().unwrap();
        let enc = rows.get(trigger.id()).and_then(|(_, enc)| enc.clone());
        rows.insert(trigger.id().clone(), (trigger.clone(), enc));
        Ok(trigger.clone())
    }
    async fn delete(&self, id: &TriggerId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
    async fn list_by_pipeline(&self, pipeline_id: &PipelineId) -> DomainResult<Vec<Trigger>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|(t, _)| t.pipeline_id() == pipeline_id)
            .map(|(t, _)| t.clone())
            .collect())
    }
    async fn list_unscheduled_cron(&self) -> DomainResult<Vec<Trigger>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|(t, _)| t.next_fire_at().is_none())
            .map(|(t, _)| t.clone())
            .collect())
    }
    async fn claim_due_cron(
        &self,
        now: DateTime<Utc>,
        _: i64,
        compute_next: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
    ) -> DomainResult<Vec<Trigger>> {
        let mut claimed = Vec::new();
        for (trigger, _) in self.rows.lock().unwrap().values_mut() {
            if trigger.is_enabled() && trigger.next_fire_at().is_some_and(|at| at <= now) {
                trigger.set_next_fire_at(Some(compute_next(trigger)?));
                claimed.push(trigger.clone());
            }
        }
        Ok(claimed)
    }
}

#[derive(Default)]
pub(super) struct StubApps {
    pub(super) apps: Mutex<Vec<App>>,
    grants: Mutex<Vec<Grant>>,
}

#[async_trait]
impl AppRepository for StubApps {
    async fn create_app(&self, _: &App, _: &AppCredential) -> DomainResult<()> {
        unreachable!("the runner app is provisioned with its grant")
    }
    async fn provision_agent(
        &self,
        _: &App,
        _: &AppCredential,
        _: &Agent,
        _: &Grant,
    ) -> DomainResult<()> {
        unreachable!("no agent in a trigger action")
    }
    async fn provision(&self, app: &App, _: &AppCredential, grant: &Grant) -> DomainResult<()> {
        self.apps.lock().unwrap().push(app.clone());
        self.grants.lock().unwrap().push(grant.clone());
        Ok(())
    }
    async fn find_by_id(&self, id: &AppId) -> DomainResult<App> {
        Err(DomainError::not_found("App", id.to_string()))
    }
    async fn list_by_organization(
        &self,
        organization_id: &OrganizationId,
    ) -> DomainResult<Vec<App>> {
        Ok(self
            .apps
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.organization_id() == organization_id)
            .cloned()
            .collect())
    }
    async fn set_active(&self, _: &AppId, _: bool) -> DomainResult<()> {
        unreachable!("no app update in a trigger action")
    }
    async fn delete(&self, _: &AppId) -> DomainResult<()> {
        unreachable!("no app delete in a trigger action")
    }
}

struct StubCipher;

impl SecretCipher for StubCipher {
    fn encrypt(&self, plaintext: &str) -> DomainResult<Vec<u8>> {
        Ok(plaintext.as_bytes().iter().rev().copied().collect())
    }
    fn decrypt(&self, _: &[u8]) -> DomainResult<String> {
        unreachable!("no decrypt in a trigger action")
    }
}

struct StubSchedule;

impl CronSchedule for StubSchedule {
    fn next_after(&self, _: &str, after: DateTime<Utc>) -> DomainResult<DateTime<Utc>> {
        Ok(after + chrono::Duration::hours(1))
    }
}

/// Grants everything but `runPipeline`: a trigger manager who may not run the pipeline.
#[derive(Default)]
struct NoRunPermissionService {
    inner: RecordingPermissionService,
}

#[async_trait]
impl PermissionService for NoRunPermissionService {
    async fn check(&self, caller: &CallerContext, permission: Permission) -> DomainResult<()> {
        let refused = permission.key() == "runPipeline";
        self.inner.check(caller, permission).await?;
        if refused {
            return Err(DomainError::forbidden("no run rights"));
        }
        Ok(())
    }
}

pub(super) struct Lab {
    pub(super) actions: Arc<Actions>,
    pub(super) uc: Arc<TriggerUseCases>,
    triggers: Arc<StubTriggers>,
    pub(super) apps: Arc<StubApps>,
    policy: Arc<CountingPolicy>,
    pipelines: Arc<PipelineUseCases>,
    pub(super) jobs: Arc<StubJobs>,
}

impl Lab {
    pub(super) async fn create(&self, source: TriggerSource) -> DomainResult<CreatedTrigger> {
        self.actions.run(&*self.uc, &alice(), create(source)).await
    }

    async fn update(&self, id: &TriggerId) -> DomainResult<Trigger> {
        self.actions
            .run(
                &*self.uc,
                &alice(),
                UpdateTrigger {
                    id: id.clone(),
                    name: TriggerName::new("hourly").unwrap(),
                    source: cron(),
                    inputs: Vec::new(),
                },
            )
            .await
    }

    async fn delete(&self, id: &TriggerId) -> DomainResult<Deleted<Trigger>> {
        self.actions
            .run(&*self.uc, &alice(), DeleteTrigger { id: id.clone() })
            .await
    }

    pub(super) fn firer(&self) -> Arc<TriggerFirer> {
        Arc::new(TriggerFirer::new(
            self.actions.clone(),
            self.uc.clone(),
            self.pipelines.clone(),
        ))
    }

    pub(super) fn stored(&self, id: &TriggerId) -> Trigger {
        self.triggers.rows.lock().unwrap()[id].0.clone()
    }

    pub(super) fn seed(&self) -> Trigger {
        let trigger = Trigger::create(
            pipeline_id(),
            TriggerName::new("nightly").unwrap(),
            cron(),
            Vec::new(),
        )
        .unwrap();
        self.triggers
            .rows
            .lock()
            .unwrap()
            .insert(trigger.id().clone(), (trigger.clone(), None));
        trigger
    }
}

pub(super) fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let project = ProjectBuilder::for_org_id(organization_id(), "rocket")
        .id(ProjectId::new("proj-1"))
        .build();
    let pipeline = PipelineBuilder::for_project_id(project.id().clone())
        .id(pipeline_id())
        .build();
    let triggers = Arc::new(StubTriggers::default());
    let apps = Arc::new(StubApps::default());
    let policy = Arc::new(CountingPolicy::default());
    let jobs = Arc::new(StubJobs::default());
    let pipeline_repo = Arc::new(OnePipeline(pipeline));
    let dispatch = Arc::new(DispatchUseCases::new(
        Arc::new(StubRegistry::default()),
        permissions.clone(),
        jobs.clone(),
        pipeline_repo.clone(),
        Arc::new(EchoResolver),
    ));
    let project_repo = Arc::new(OneProject(project));
    Lab {
        actions: Arc::new(actions(permissions)),
        uc: Arc::new(TriggerUseCases::new(
            triggers.clone(),
            pipeline_repo.clone(),
            project_repo.clone(),
            apps.clone(),
            Arc::new(StubHash::secrets()),
            policy.clone(),
            Arc::new(StubCipher),
            Arc::new(StubSchedule),
        )),
        triggers,
        apps,
        policy,
        pipelines: Arc::new(PipelineUseCases::new(
            pipeline_repo,
            project_repo,
            jobs.clone(),
            Arc::new(EchoResolver),
            dispatch,
        )),
        jobs,
    }
}

fn organization_id() -> OrganizationId {
    OrganizationId::new("acme")
}

pub(super) fn pipeline_id() -> PipelineId {
    PipelineId::new("pl-1")
}

pub(super) fn cron() -> TriggerSource {
    TriggerSource::Cron(CronSpec::new("0 3 * * *").unwrap())
}

fn webhook() -> TriggerSource {
    TriggerSource::Webhook(WebhookSpec::new(None).unwrap())
}

fn create(source: TriggerSource) -> CreateTrigger {
    CreateTrigger {
        pipeline_id: pipeline_id(),
        name: TriggerName::new("nightly").unwrap(),
        source,
        inputs: Vec::new(),
    }
}

#[tokio::test]
async fn a_create_checks_manage_then_run_and_provisions_the_runner_app_once() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let CreatedTrigger {
        trigger,
        webhook_secret: secret,
    } = lab.create(cron()).await.unwrap();
    lab.create(cron()).await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageTriggers(pipeline_id()),
            Permission::RunPipeline(pipeline_id()),
            Permission::ManageTriggers(pipeline_id()),
            Permission::RunPipeline(pipeline_id()),
        ]
    );
    assert!(secret.is_none());
    assert!(trigger.next_fire_at().is_some());
    assert_eq!(lab.triggers.rows.lock().unwrap().len(), 2);
    let apps = lab.apps.apps.lock().unwrap();
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name().to_string(), TRIGGER_RUNNER_APP_NAME);
    assert_eq!(apps[0].organization_id(), &organization_id());
    assert_eq!(lab.policy.reloads(), 1);
}

#[tokio::test]
async fn a_webhook_create_answers_the_plaintext_once_and_stores_it_encrypted() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let CreatedTrigger {
        trigger,
        webhook_secret: secret,
    } = lab.create(webhook()).await.unwrap();

    let secret = secret.expect("a webhook trigger has a secret");
    assert_eq!(secret.len(), 64);
    assert!(trigger.next_fire_at().is_none());
    assert_eq!(
        lab.triggers.webhook_secret(trigger.id()).await.unwrap(),
        Some(StubCipher.encrypt(&secret).unwrap())
    );
}

#[tokio::test]
async fn a_denied_caller_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create(cron()).await.err().unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
    assert!(lab.apps.apps.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_manager_without_run_rights_cannot_create_a_trigger() {
    let permissions = Arc::new(NoRunPermissionService::default());
    let lab = lab(permissions.clone());

    let err = lab.create(cron()).await.err().unwrap();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(
        permissions.inner.permissions(),
        vec![
            Permission::ManageTriggers(pipeline_id()),
            Permission::RunPipeline(pipeline_id()),
        ]
    );
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
    assert!(lab.apps.apps.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_create_on_an_unknown_pipeline_is_not_found_and_writes_nothing() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab
        .actions
        .run(
            &*lab.uc,
            &alice(),
            CreateTrigger {
                pipeline_id: PipelineId::new("ghost"),
                ..create(cron())
            },
        )
        .await
        .err()
        .unwrap();

    assert!(matches!(err, DomainError::NotFound { .. }));
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
    assert!(lab.apps.apps.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_pipeline_list_checks_manage_triggers() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let triggers = lab
        .actions
        .run(
            &*lab.uc,
            &alice(),
            ListPipelineTriggers {
                pipeline_id: pipeline_id(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageTriggers(pipeline_id())]
    );
    assert_eq!(triggers.len(), 1);
    assert_eq!(triggers[0].id(), seeded.id());
}

#[tokio::test]
async fn a_get_checks_manage_on_the_trigger() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let trigger = lab
        .actions
        .run(
            &*lab.uc,
            &alice(),
            GetTrigger {
                id: seeded.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(trigger.id(), seeded.id());
    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageTrigger(seeded.id().clone())]
    );
}

#[tokio::test]
async fn an_update_checks_manage_and_run_on_the_trigger() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let updated = lab.update(seeded.id()).await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageTrigger(seeded.id().clone()),
            Permission::RunTriggerPipeline(seeded.id().clone()),
        ]
    );
    assert_eq!(updated.name().as_str(), "hourly");
    assert!(updated.next_fire_at().is_some());
}

#[tokio::test]
async fn a_manager_without_run_rights_cannot_update_a_trigger() {
    let lab = lab(Arc::new(NoRunPermissionService::default()));
    let seeded = lab.seed();

    let err = lab.update(seeded.id()).await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    let rows = lab.triggers.rows.lock().unwrap();
    assert_eq!(rows[seeded.id()].0.name().as_str(), "nightly");
}

#[tokio::test]
async fn a_disable_then_a_delete_check_manage_on_the_trigger() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let disabled = lab
        .actions
        .run(
            &*lab.uc,
            &alice(),
            SetTriggerEnabled {
                id: seeded.id().clone(),
                enabled: false,
            },
        )
        .await
        .unwrap();
    let deleted = lab.delete(seeded.id()).await.unwrap();

    assert!(!disabled.is_enabled());
    assert_eq!(deleted.last_state().id(), seeded.id());
    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageTrigger(seeded.id().clone()),
            Permission::ManageTrigger(seeded.id().clone()),
        ]
    );
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_denied_delete_never_reads_or_removes() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let seeded = lab.seed();

    let err = lab.delete(seeded.id()).await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.triggers.rows.lock().unwrap().contains_key(seeded.id()));
}

#[tokio::test]
async fn an_allowed_action_on_an_unknown_trigger_is_not_found() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab.delete(&TriggerId::new("missing")).await.unwrap_err();

    assert!(matches!(err, DomainError::NotFound { .. }));
}

fn cron_scheduler() -> CallerContext {
    CallerContext::Service(ServiceIdentity::cron_scheduler())
}

#[derive(Default)]
struct StubFiring {
    fired: Mutex<Vec<TriggerId>>,
    fail: Vec<TriggerId>,
}

#[async_trait]
impl TriggerFiring for StubFiring {
    async fn fire(
        &self,
        trigger_id: &TriggerId,
        _: Option<&serde_json::Value>,
        _: Option<&str>,
    ) -> DomainResult<Job> {
        self.fired.lock().unwrap().push(trigger_id.clone());
        if self.fail.contains(trigger_id) {
            return Err(DomainError::internal("boom"));
        }
        let pipeline = PipelineBuilder::for_project_id(ProjectId::new("proj-1")).build();
        Ok(crate::test_support::jobs::job(&pipeline))
    }
}

impl Lab {
    fn due(&self) -> Trigger {
        let mut trigger = self.seed();
        trigger.set_next_fire_at(Some(clock::now() - chrono::Duration::minutes(1)));
        self.triggers
            .rows
            .lock()
            .unwrap()
            .insert(trigger.id().clone(), (trigger.clone(), None));
        trigger
    }

    fn scheduler(&self, firing: Arc<StubFiring>) -> TriggerCronScheduler {
        TriggerCronScheduler::new(self.actions.clone(), self.uc.clone(), firing)
    }
}

#[tokio::test]
async fn a_tick_seeds_the_unscheduled_then_fires_every_due_trigger_with_no_permission_asked() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let fresh = lab.seed();
    let a = lab.due();
    let b = lab.due();
    let firing = Arc::new(StubFiring::default());

    assert_eq!(lab.scheduler(firing.clone()).tick().await, 2);

    assert!(lab.stored(fresh.id()).next_fire_at().is_some(), "seeded");
    let mut fired = firing.fired.lock().unwrap().clone();
    fired.sort_by_key(ToString::to_string);
    let mut due = vec![a.id().clone(), b.id().clone()];
    due.sort_by_key(ToString::to_string);
    assert_eq!(fired, due);
    assert!(
        lab.stored(a.id()).next_fire_at().unwrap() > clock::now(),
        "advanced"
    );
    assert!(permissions.checks().is_empty());
}

#[tokio::test]
async fn one_fire_failure_does_not_abort_the_pass() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let a = lab.due();
    lab.due();
    let firing = Arc::new(StubFiring {
        fail: vec![a.id().clone()],
        ..StubFiring::default()
    });

    assert_eq!(lab.scheduler(firing.clone()).tick().await, 1);
    assert_eq!(firing.fired.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_webhook_trigger_is_never_seeded_as_cron() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let hook = lab.create(webhook()).await.unwrap().trigger;

    lab.scheduler(Arc::default()).tick().await;

    assert!(lab.stored(hook.id()).next_fire_at().is_none());
}

#[tokio::test]
async fn the_cron_passes_refuse_a_caller_that_is_not_a_service() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let fresh = lab.seed();

    let seed = lab
        .actions
        .run(&*lab.uc, &alice(), ScheduleCronTriggers)
        .await
        .unwrap_err();
    let claim = ClaimDueTriggers {
        now: clock::now(),
        limit: 10,
    };
    let claim = lab
        .actions
        .run(&*lab.uc, &alice(), claim)
        .await
        .unwrap_err();
    let scheduled = lab
        .actions
        .run(&*lab.uc, &cron_scheduler(), ScheduleCronTriggers)
        .await
        .unwrap();

    assert!(matches!(seed, DomainError::Forbidden(_)));
    assert!(matches!(claim, DomainError::Forbidden(_)));
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].id(), fresh.id());
}
