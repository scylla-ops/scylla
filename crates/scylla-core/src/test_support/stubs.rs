use crate::application::agent::{AgentDispatch, DispatchNode, JobDispatch};
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{
    HashService, JobRepository, PipelineRepository, ProjectRepository, SecretResolver,
    SessionRepository, SignupRepository, UserRepository,
};
use crate::domain::app::{AppSecret, AppSecretHash};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId, UserId};
use crate::domain::job::{Job, JobStatus};
use crate::domain::organization::Organization;
use crate::domain::pipeline::{Pipeline, PipelineNode};
use crate::domain::project::Project;
use crate::domain::session::Session;
use crate::domain::user::{Email, Password, PasswordHash, User, Username};
use async_trait::async_trait;
use scylla_auth::authz::{
    Grant, GrantRepository, PolicyControl, Principal, Role, RoleRepository, Scope, Visibility,
};
use std::collections::HashMap;
use std::sync::Mutex;

const HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$abc$def";

pub fn alice() -> CallerContext {
    CallerContext::User(UserId::new("alice"))
}

pub fn empty_page<T>() -> DomainResult<PaginatedResult<T>> {
    Ok(PaginatedResult::new(
        Vec::new(),
        &PaginationParams::default(),
        0,
    ))
}

#[derive(Default)]
pub struct CountingPolicy {
    reloads: Mutex<usize>,
}

impl CountingPolicy {
    pub fn reloads(&self) -> usize {
        *self.reloads.lock().unwrap()
    }
}

#[async_trait]
impl PolicyControl for CountingPolicy {
    async fn reload(&self) -> DomainResult<()> {
        *self.reloads.lock().unwrap() += 1;
        Ok(())
    }
}

enum Hashes {
    Passwords,
    Secrets,
}

pub struct StubHash {
    kind: Hashes,
    hashed: Mutex<usize>,
}

impl StubHash {
    pub fn passwords() -> Self {
        Self {
            kind: Hashes::Passwords,
            hashed: Mutex::default(),
        }
    }

    pub fn secrets() -> Self {
        Self {
            kind: Hashes::Secrets,
            hashed: Mutex::default(),
        }
    }

    pub fn hashed(&self) -> usize {
        *self.hashed.lock().unwrap()
    }
}

#[async_trait]
impl HashService for StubHash {
    async fn hash(&self, _: &Password) -> DomainResult<PasswordHash> {
        assert!(
            matches!(self.kind, Hashes::Passwords),
            "no password in this action"
        );
        *self.hashed.lock().unwrap() += 1;
        PasswordHash::new(HASH)
    }
    async fn verify(&self, _: &Password, _: &PasswordHash) -> DomainResult<bool> {
        unreachable!("no password check in this action")
    }
    async fn hash_secret(&self, _: &AppSecret) -> DomainResult<AppSecretHash> {
        assert!(
            matches!(self.kind, Hashes::Secrets),
            "no app secret in this action"
        );
        *self.hashed.lock().unwrap() += 1;
        AppSecretHash::new(HASH)
    }
    async fn verify_secret(&self, _: &AppSecret, _: &AppSecretHash) -> DomainResult<bool> {
        unreachable!("no secret check in this action")
    }
}

/// The default registry fails a test that dispatches; `accepting` records each dispatch and
/// counts it as a load until `release`.
#[derive(Default)]
pub struct StubRegistry {
    accepts: bool,
    connected: Mutex<Vec<AppId>>,
    disconnected: Mutex<Vec<AppId>>,
    dispatched: Mutex<Vec<(AppId, JobDispatch)>>,
    loads: Mutex<HashMap<AppId, usize>>,
}

impl StubRegistry {
    pub fn accepting() -> Self {
        Self {
            accepts: true,
            ..Self::default()
        }
    }

    pub fn connect(&self, app_id: &AppId) {
        self.connected.lock().unwrap().push(app_id.clone());
    }

    pub fn load(&self, app_id: &AppId, in_flight: usize) {
        self.loads.lock().unwrap().insert(app_id.clone(), in_flight);
    }

    pub fn disconnected(&self) -> Vec<AppId> {
        self.disconnected.lock().unwrap().clone()
    }

    pub fn dispatched(&self) -> Vec<(AppId, JobDispatch)> {
        self.dispatched.lock().unwrap().clone()
    }

    pub fn dispatched_to(&self) -> Vec<AppId> {
        self.dispatched().into_iter().map(|(id, _)| id).collect()
    }
}

#[async_trait]
impl AgentDispatch for StubRegistry {
    fn connected(&self) -> Vec<AppId> {
        self.connected.lock().unwrap().clone()
    }
    async fn dispatch(&self, app_id: &AppId, dispatch: &JobDispatch) -> DomainResult<()> {
        assert!(self.accepts, "no dispatch in this action");
        self.dispatched
            .lock()
            .unwrap()
            .push((app_id.clone(), dispatch.clone()));
        *self
            .loads
            .lock()
            .unwrap()
            .entry(app_id.clone())
            .or_insert(0) += 1;
        Ok(())
    }
    fn disconnect(&self, app_id: &AppId) {
        self.disconnected.lock().unwrap().push(app_id.clone());
    }
    fn in_flight(&self, app_id: &AppId) -> usize {
        self.loads.lock().unwrap().get(app_id).copied().unwrap_or(0)
    }
    fn release(&self, app_id: &AppId) {
        if let Some(n) = self.loads.lock().unwrap().get_mut(app_id) {
            *n = n.saturating_sub(1);
        }
    }
}

/// Lists its pending rows that no agent holds, records each attribution, and orphans a fixed
/// count.
#[derive(Default)]
pub struct StubJobs {
    rows: Mutex<Vec<Job>>,
    assigned: Mutex<Vec<(JobId, AppId)>>,
    swept: Mutex<Vec<Vec<AppId>>>,
    orphans: u64,
}

impl StubJobs {
    pub fn with(rows: Vec<Job>) -> Self {
        Self {
            rows: Mutex::new(rows),
            ..Self::default()
        }
    }

    pub fn orphaning(orphans: u64) -> Self {
        Self {
            orphans,
            ..Self::default()
        }
    }

    pub fn rows(&self) -> Vec<Job> {
        self.rows.lock().unwrap().clone()
    }

    pub fn assigned(&self) -> Vec<(JobId, AppId)> {
        self.assigned.lock().unwrap().clone()
    }

    pub fn swept(&self) -> Vec<Vec<AppId>> {
        self.swept.lock().unwrap().clone()
    }
}

#[async_trait]
impl JobRepository for StubJobs {
    async fn create(&self, job: &Job) -> DomainResult<Job> {
        self.rows.lock().unwrap().push(job.clone());
        Ok(job.clone())
    }
    async fn find_by_id(&self, id: &JobId) -> DomainResult<Job> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .find(|j| j.id() == id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Job", id.to_string()))
    }
    async fn update(&self, _: &Job) -> DomainResult<Job> {
        unreachable!("no job update in this action")
    }
    async fn set_agent(&self, job_id: &JobId, app_id: &AppId) -> DomainResult<()> {
        if let Some(job) = self
            .rows
            .lock()
            .unwrap()
            .iter_mut()
            .find(|j| j.id() == job_id)
        {
            job.assign_agent(app_id.clone());
        }
        self.assigned
            .lock()
            .unwrap()
            .push((job_id.clone(), app_id.clone()));
        Ok(())
    }
    async fn list_pending_unassigned(&self) -> DomainResult<Vec<Job>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|j| j.status() == JobStatus::Pending && j.agent_app_id().is_none())
            .cloned()
            .collect())
    }
    async fn orphan_running_without_agents(&self, connected: &[AppId]) -> DomainResult<u64> {
        self.swept.lock().unwrap().push(connected.to_vec());
        Ok(self.orphans)
    }
    async fn delete(&self, _: &JobId) -> DomainResult<()> {
        unreachable!("no job delete in this action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Job>> {
        empty_page()
    }
    async fn list_by_pipeline(
        &self,
        _: &PipelineId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty_page()
    }
    async fn list_by_project(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty_page()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Job>> {
        empty_page()
    }
}

pub struct EchoResolver;

#[async_trait]
impl SecretResolver for EchoResolver {
    async fn resolve(
        &self,
        _: &ProjectId,
        nodes: &[PipelineNode],
    ) -> DomainResult<Vec<DispatchNode>> {
        Ok(nodes
            .iter()
            .map(|n| DispatchNode {
                id: n.id().to_string(),
                deps: n.deps().iter().map(ToString::to_string).collect(),
                working_dir: n.working_dir().map(|w| w.as_str().to_string()),
                step: n.step().clone(),
                env: Vec::new(),
            })
            .collect())
    }
}

pub struct NoUsers;

#[async_trait]
impl UserRepository for NoUsers {
    async fn create(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        Err(DomainError::not_found("User", id.to_string()))
    }
    async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
        Ok(Vec::new())
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        Err(DomainError::not_found("User", username.to_string()))
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        Err(DomainError::not_found("User", email.to_string()))
    }
    async fn update(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn delete(&self, _: &UserId) -> DomainResult<()> {
        unreachable!("no user write in this action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        empty_page()
    }
    async fn username_exists(&self, _: &Username) -> DomainResult<bool> {
        Ok(false)
    }
}

pub struct OneProject(pub Project);

#[async_trait]
impl ProjectRepository for OneProject {
    async fn create(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in this action")
    }
    async fn provision_with_owner(&self, _: &Project, _: &Grant) -> DomainResult<()> {
        unreachable!("no project write in this action")
    }
    async fn list_principals(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        empty_page()
    }
    async fn list_for_user(
        &self,
        _: &UserId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("Project", id.to_string()))
        }
    }
    async fn update(&self, _: &Project) -> DomainResult<Project> {
        unreachable!("no project write in this action")
    }
    async fn delete(&self, _: &Project) -> DomainResult<()> {
        unreachable!("no project write in this action")
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
        _: &Visibility,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
}

pub struct OnePipeline(pub Pipeline);

#[async_trait]
impl PipelineRepository for OnePipeline {
    async fn create(&self, _: &Pipeline) -> DomainResult<Pipeline> {
        unreachable!("no pipeline write in this action")
    }
    async fn find_by_id(&self, id: &PipelineId) -> DomainResult<Pipeline> {
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("Pipeline", id.to_string()))
        }
    }
    async fn update(&self, _: &Pipeline) -> DomainResult<Pipeline> {
        unreachable!("no pipeline write in this action")
    }
    async fn delete(&self, _: &PipelineId) -> DomainResult<()> {
        unreachable!("no pipeline write in this action")
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty_page()
    }
    async fn list_by_project(
        &self,
        _: &ProjectId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty_page()
    }
    async fn list_by_organization(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Pipeline>> {
        empty_page()
    }
}

#[derive(Default)]
pub struct StubRoles {
    rows: Mutex<Vec<Role>>,
}

impl StubRoles {
    pub fn new(rows: Vec<Role>) -> Self {
        Self {
            rows: Mutex::new(rows),
        }
    }

    pub fn rows(&self) -> Vec<Role> {
        self.rows.lock().unwrap().clone()
    }
}

#[async_trait]
impl RoleRepository for StubRoles {
    async fn list_all(&self) -> DomainResult<Vec<Role>> {
        Ok(self.rows())
    }
    async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned())
    }
    async fn create(&self, role: &Role) -> DomainResult<()> {
        self.rows.lock().unwrap().push(role.clone());
        Ok(())
    }
    async fn update(&self, role: &Role) -> DomainResult<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(row) = rows.iter_mut().find(|r| r.id == role.id) {
            *row = role.clone();
        }
        Ok(())
    }
    async fn delete(&self, id: &str) -> DomainResult<()> {
        self.rows.lock().unwrap().retain(|r| r.id != id);
        Ok(())
    }
}

#[derive(Default)]
pub struct StubGrants {
    rows: Vec<Grant>,
    created: Mutex<Vec<Grant>>,
    deleted: Mutex<Vec<String>>,
}

impl StubGrants {
    pub fn new(rows: Vec<Grant>) -> Self {
        Self {
            rows,
            created: Mutex::default(),
            deleted: Mutex::default(),
        }
    }

    pub fn created(&self) -> Vec<Grant> {
        self.created.lock().unwrap().clone()
    }

    pub fn deleted(&self) -> Vec<String> {
        self.deleted.lock().unwrap().clone()
    }
}

#[async_trait]
impl GrantRepository for StubGrants {
    async fn list_all(&self) -> DomainResult<Vec<Grant>> {
        Ok(self.rows.clone())
    }
    async fn create(&self, grant: &Grant) -> DomainResult<()> {
        self.created.lock().unwrap().push(grant.clone());
        Ok(())
    }
    async fn delete(&self, id: &str) -> DomainResult<()> {
        self.deleted.lock().unwrap().push(id.to_string());
        Ok(())
    }
    async fn revoke_all(&self, _: &Principal, _: &Scope) -> DomainResult<u64> {
        Ok(0)
    }
}

pub struct OneUser(pub User);

#[async_trait]
impl UserRepository for OneUser {
    async fn create(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("User", id.to_string()))
        }
    }
    async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
        Ok(vec![self.0.clone()])
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        if username == self.0.username() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("User", username.to_string()))
        }
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        if Some(email) == self.0.email() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("User", email.to_string()))
        }
    }
    async fn update(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn delete(&self, _: &UserId) -> DomainResult<()> {
        unreachable!("no user write in this action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        empty_page()
    }
    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        Ok(username == self.0.username())
    }
}

#[derive(Default)]
pub struct StubSessions {
    rows: Mutex<Vec<Session>>,
    deleted: Mutex<Vec<String>>,
}

impl StubSessions {
    pub fn with(session: Session) -> Self {
        Self {
            rows: Mutex::new(vec![session]),
            deleted: Mutex::default(),
        }
    }

    pub fn rows(&self) -> Vec<Session> {
        self.rows.lock().unwrap().clone()
    }

    pub fn deleted(&self) -> Vec<String> {
        self.deleted.lock().unwrap().clone()
    }
}

#[async_trait]
impl SessionRepository for StubSessions {
    async fn create(&self, session: &Session) -> DomainResult<Session> {
        self.rows.lock().unwrap().push(session.clone());
        Ok(session.clone())
    }
    async fn find_by_token(&self, token: &str) -> DomainResult<Session> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.token() == token)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Session", token))
    }
    async fn delete_by_token(&self, token: &str) -> DomainResult<()> {
        self.rows.lock().unwrap().retain(|s| s.token() != token);
        self.deleted.lock().unwrap().push(token.to_string());
        Ok(())
    }
    async fn delete_expired(&self) -> DomainResult<u64> {
        let mut rows = self.rows.lock().unwrap();
        let before = rows.len();
        rows.retain(|s| !s.is_expired());
        Ok((before - rows.len()) as u64)
    }
}

/// Each provisioned account, with the provider identity when there is one.
#[derive(Default)]
pub struct StubSignups {
    provisioned: Mutex<Vec<(User, Organization, Grant, Option<String>)>>,
}

impl StubSignups {
    pub fn provisioned(&self) -> Vec<(User, Organization, Grant, Option<String>)> {
        self.provisioned.lock().unwrap().clone()
    }
}

#[async_trait]
impl SignupRepository for StubSignups {
    async fn provision_account(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()> {
        self.provisioned.lock().unwrap().push((
            user.clone(),
            organization.clone(),
            grant.clone(),
            None,
        ));
        Ok(())
    }
    async fn provision_account_with_identity(
        &self,
        user: &User,
        organization: &Organization,
        grant: &Grant,
        _: &str,
        provider_user_id: &str,
    ) -> DomainResult<()> {
        self.provisioned.lock().unwrap().push((
            user.clone(),
            organization.clone(),
            grant.clone(),
            Some(provider_user_id.to_string()),
        ));
        Ok(())
    }
}
