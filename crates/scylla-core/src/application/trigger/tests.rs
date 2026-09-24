//! The trigger's actions through the engine, on stub ports.

use super::*;
use crate::application::PermissionAuthorizer;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::agent::Agent;
use crate::domain::app::{AppSecret, AppSecretHash};
use crate::domain::ids::{AppId, PipelineId, ProjectId, UserId};
use crate::domain::pipeline::Pipeline;
use crate::domain::project::Project;
use crate::domain::trigger::{CronSpec, WebhookSpec};
use crate::domain::user::{Password, PasswordHash};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService};
use crate::test_support::pipelines::PipelineBuilder;
use crate::test_support::projects::ProjectBuilder;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_auth::authz::Visibility;
use scylla_extension::{Actions, Hooks};
use std::collections::HashMap;
use std::sync::Mutex;

fn empty<T>() -> DomainResult<PaginatedResult<T>> {
    Ok(PaginatedResult::new(
        Vec::new(),
        &PaginationParams::default(),
        0,
    ))
}

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
        unreachable!("no scheduler in a trigger action")
    }
    async fn claim_due_cron(
        &self,
        _: DateTime<Utc>,
        _: i64,
        _: &(dyn for<'a> Fn(&'a Trigger) -> DomainResult<DateTime<Utc>> + Sync),
    ) -> DomainResult<Vec<Trigger>> {
        unreachable!("no scheduler in a trigger action")
    }
}

struct StubPipelines {
    pipeline: Pipeline,
}

#[async_trait]
impl PipelineRepository for StubPipelines {
    async fn create(&self, _: &Pipeline) -> DomainResult<Pipeline> {
        unreachable!("no pipeline write in a trigger action")
    }
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        if id == self.pipeline.id() {
            Ok(self.pipeline.clone())
        } else {
            Err(DomainError::not_found("Pipeline", id.to_string()))
        }
    }
    async fn update(&self, _: &Pipeline) -> DomainResult<Pipeline> {
        unreachable!("no pipeline write in a trigger action")
    }
    async fn delete(&self, _: &PipelineId) -> DomainResult<()> {
        unreachable!("no pipeline write in a trigger action")
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty()
    }
    async fn list_by_project(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty()
    }
}

struct StubProjects {
    project: Project,
}

#[async_trait]
impl ProjectRepository for StubProjects {
    async fn create(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in a trigger action")
    }
    async fn provision_with_owner(&self, _: &Project, _: &Grant) -> DomainResult<()> {
        unreachable!("no project write in a trigger action")
    }
    async fn list_principals(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        empty()
    }
    async fn list_for_user(
        &self,
        _: &UserId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        if id == self.project.id() {
            Ok(self.project.clone())
        } else {
            Err(DomainError::not_found("Project", id.to_string()))
        }
    }
    async fn find_by_ids(&self, _: &[ProjectId]) -> DomainResult<Vec<Project>> {
        Ok(Vec::new())
    }
    async fn update(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in a trigger action")
    }
    async fn delete(&self, _: &Project) -> DomainResult<()> {
        unreachable!("no project write in a trigger action")
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn list_active(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
        _: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty()
    }
}

#[derive(Default)]
struct StubApps {
    apps: Mutex<Vec<App>>,
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

struct StubHash;

#[async_trait]
impl HashService for StubHash {
    async fn hash(&self, _: &Password) -> DomainResult<PasswordHash> {
        unreachable!("no password in a trigger action")
    }
    async fn verify(&self, _: &Password, _: &PasswordHash) -> DomainResult<bool> {
        unreachable!("no password in a trigger action")
    }
    async fn hash_secret(&self, _: &AppSecret) -> DomainResult<AppSecretHash> {
        AppSecretHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def")
    }
    async fn verify_secret(&self, _: &AppSecret, _: &AppSecretHash) -> DomainResult<bool> {
        unreachable!("no secret check in a trigger action")
    }
}

#[derive(Default)]
struct StubPolicy {
    reloads: Mutex<usize>,
}

#[async_trait]
impl PolicyControl for StubPolicy {
    async fn reload(&self) -> DomainResult<()> {
        *self.reloads.lock().unwrap() += 1;
        Ok(())
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

/// Grants everything but `RunPipeline`: a trigger manager who may not run the pipeline.
#[derive(Default)]
struct NoRunPermissionService {
    inner: RecordingPermissionService,
}

#[async_trait]
impl PermissionService for NoRunPermissionService {
    async fn check(&self, caller: &CallerContext, permission: Permission) -> DomainResult<()> {
        let refused = matches!(permission, Permission::RunPipeline(_));
        self.inner.check(caller, permission).await?;
        if refused {
            return Err(DomainError::forbidden("no run rights"));
        }
        Ok(())
    }
}

type Uc<PS> =
    TriggerUseCases<StubTriggers, StubPipelines, StubProjects, StubApps, StubHash, StubPolicy, PS>;

struct Lab<PS: PermissionService> {
    actions: Actions,
    uc: Uc<PS>,
    triggers: Arc<StubTriggers>,
    apps: Arc<StubApps>,
    policy: Arc<StubPolicy>,
}

impl<PS: PermissionService> Lab<PS> {
    async fn create(&self, source: TriggerSource) -> DomainResult<(Trigger, Option<String>)> {
        self.actions.run(&self.uc, &alice(), create(source)).await
    }

    fn seed(&self) -> Trigger {
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

fn lab<PS: PermissionService + 'static>(permissions: Arc<PS>) -> Lab<PS> {
    let project = ProjectBuilder::for_org_id(organization_id(), "rocket")
        .id(ProjectId::new("proj-1"))
        .build();
    let pipeline = PipelineBuilder::for_project_id(project.id().clone())
        .id(pipeline_id())
        .build();
    let triggers = Arc::new(StubTriggers::default());
    let apps = Arc::new(StubApps::default());
    let policy = Arc::new(StubPolicy::default());
    Lab {
        actions: Actions::new(
            Arc::new(PermissionAuthorizer::new(permissions.clone())),
            Arc::new(Hooks::new()),
        ),
        uc: TriggerUseCases::new(
            triggers.clone(),
            Arc::new(StubPipelines { pipeline }),
            Arc::new(StubProjects { project }),
            apps.clone(),
            Arc::new(StubHash),
            policy.clone(),
            permissions,
            Arc::new(StubCipher),
            Arc::new(StubSchedule),
        ),
        triggers,
        apps,
        policy,
    }
}

fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
}

fn organization_id() -> OrganizationId {
    OrganizationId::new("acme")
}

fn pipeline_id() -> PipelineId {
    PipelineId::new("pl-1")
}

fn cron() -> TriggerSource {
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

    let (trigger, secret) = lab.create(cron()).await.unwrap();
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
    assert_eq!(*lab.policy.reloads.lock().unwrap(), 1);
}

#[tokio::test]
async fn a_webhook_create_answers_the_plaintext_once_and_stores_it_encrypted() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let (trigger, secret) = lab.create(webhook()).await.unwrap();

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

    let err = lab.create(cron()).await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
    assert!(lab.apps.apps.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_manager_without_run_rights_cannot_create_a_trigger() {
    let permissions = Arc::new(NoRunPermissionService::default());
    let lab = lab(permissions.clone());

    let err = lab.create(cron()).await.unwrap_err();

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
            &lab.uc,
            &alice(),
            CreateTrigger {
                pipeline_id: PipelineId::new("ghost"),
                ..create(cron())
            },
        )
        .await
        .unwrap_err();

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
            &lab.uc,
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
async fn an_update_checks_manage_and_run_on_the_loaded_triggers_pipeline() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let updated = lab
        .uc
        .update(
            &alice(),
            seeded.id(),
            TriggerName::new("hourly").unwrap(),
            cron(),
            Vec::new(),
        )
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageTriggers(pipeline_id()),
            Permission::RunPipeline(pipeline_id()),
        ]
    );
    assert_eq!(updated.name().as_str(), "hourly");
    assert!(updated.next_fire_at().is_some());
}

#[tokio::test]
async fn a_disable_then_a_delete_check_manage_triggers() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let seeded = lab.seed();

    let disabled = lab
        .uc
        .set_enabled(&alice(), seeded.id(), false)
        .await
        .unwrap();
    lab.uc.delete(&alice(), seeded.id()).await.unwrap();

    assert!(!disabled.is_enabled());
    assert_eq!(
        permissions.permissions(),
        vec![
            Permission::ManageTriggers(pipeline_id()),
            Permission::ManageTriggers(pipeline_id()),
        ]
    );
    assert!(lab.triggers.rows.lock().unwrap().is_empty());
}
