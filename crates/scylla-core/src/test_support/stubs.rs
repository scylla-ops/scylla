use crate::application::agent::{
    AgentDispatch, AgentOrder, AgentStream, DispatchNode, JobDispatch,
};
use crate::application::job::JobScope;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::application::{
    DispatchUseCases, HashService, JobLogLiveStream, JobLogStreamPort, JobRepository,
    PipelineRepository, ProjectRepository, SecretResolver, SessionRepository, SignupRepository,
    UserRepository,
};
use crate::domain::app::{AppSecret, AppSecretHash};
use crate::domain::caller::CallerContext;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{AppId, JobId, OrganizationId, PipelineId, ProjectId, StreamId, UserId};
use crate::domain::job::{Job, JobLog, JobStatus};
use crate::domain::organization::Organization;
use crate::domain::pipeline::{NodeId, Pipeline, PipelineNode};
use crate::domain::project::Project;
use crate::domain::session::Session;
use crate::domain::user::{Email, Password, PasswordHash, User, Username};
use crate::test_support::jobs::stored;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scylla_auth::authz::{
    Grant, GrantRepository, PolicyControl, Principal, Role, RoleRepository, Scope, Visibility,
    VisibilityResolver,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// The default registry fails a test that dispatches a job; `accepting` records each order.
/// An order to a stream that is not open fails, as the registry does; `closing` fails every
/// order.
#[derive(Default)]
pub struct StubRegistry {
    accepts: bool,
    closed: bool,
    connected: Mutex<Vec<AgentStream>>,
    disconnected: Mutex<Vec<AppId>>,
    sent: Mutex<Vec<(AppId, AgentOrder)>>,
    wakes: Mutex<Vec<Option<AppId>>>,
}

impl StubRegistry {
    pub fn accepting() -> Self {
        Self {
            accepts: true,
            ..Self::default()
        }
    }

    pub fn closing() -> Self {
        Self {
            accepts: true,
            closed: true,
            ..Self::default()
        }
    }

    /// A new stream of the agent: it replaces the one the agent had.
    pub fn connect(&self, agent: &AppId) -> AgentStream {
        let stream = AgentStream {
            agent: agent.clone(),
            id: StreamId::generate(),
        };
        let mut connected = self.connected.lock().unwrap();
        connected.retain(|s| &s.agent != agent);
        connected.push(stream.clone());
        stream
    }

    pub fn disconnected(&self) -> Vec<AppId> {
        self.disconnected.lock().unwrap().clone()
    }

    pub fn dispatched(&self) -> Vec<(AppId, JobDispatch)> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(to, order)| match order {
                AgentOrder::Run(dispatch) => Some((to.clone(), dispatch.clone())),
                AgentOrder::Cancel(_) => None,
            })
            .collect()
    }

    pub fn dispatched_to(&self) -> Vec<AppId> {
        self.dispatched().into_iter().map(|(id, _)| id).collect()
    }

    pub fn cancelled(&self) -> Vec<(AppId, JobId)> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(to, order)| match order {
                AgentOrder::Cancel(job_id) => Some((to.clone(), job_id.clone())),
                AgentOrder::Run(_) => None,
            })
            .collect()
    }

    pub fn wakes(&self) -> Vec<Option<AppId>> {
        self.wakes.lock().unwrap().clone()
    }

    fn send(&self, agent: &AppId, open: bool, order: AgentOrder) -> DomainResult<()> {
        if self.closed || !open {
            return Err(DomainError::infrastructure("the agent takes no orders"));
        }
        self.sent.lock().unwrap().push((agent.clone(), order));
        Ok(())
    }
}

impl AgentDispatch for StubRegistry {
    fn connected(&self) -> Vec<AgentStream> {
        self.connected.lock().unwrap().clone()
    }
    fn run(&self, stream: &AgentStream, dispatch: JobDispatch) -> DomainResult<()> {
        assert!(self.accepts, "no dispatch in this action");
        let open = self.connected.lock().unwrap().contains(stream);
        self.send(&stream.agent, open, AgentOrder::Run(dispatch))
    }
    fn cancel(&self, agent: &AppId, job_id: &JobId) -> DomainResult<()> {
        let open = self
            .connected
            .lock()
            .unwrap()
            .iter()
            .any(|s| &s.agent == agent);
        self.send(agent, open, AgentOrder::Cancel(job_id.clone()))
    }
    fn disconnect(&self, agent: &AppId) {
        self.disconnected.lock().unwrap().push(agent.clone());
    }
    fn wake(&self, agent: Option<&AppId>) {
        self.wakes.lock().unwrap().push(agent.cloned());
    }
}

/// A versioned store in memory: each write checks and bumps the version, as the Postgres
/// store does. `claim_next` places the oldest pending job on an idle agent that sees anything,
/// `release` returns only the jobs of its stream, and `list_live` filters a pipeline scope
/// only. `list_stranded` returns every running job: the stub keeps no last contact.
#[derive(Default)]
pub struct StubJobs {
    rows: Mutex<Vec<Job>>,
    streams: Mutex<HashMap<JobId, StreamId>>,
    released: Mutex<Vec<AgentStream>>,
}

impl StubJobs {
    pub fn with(rows: Vec<Job>) -> Self {
        Self {
            rows: Mutex::new(rows),
            ..Self::default()
        }
    }

    pub fn rows(&self) -> Vec<Job> {
        self.rows.lock().unwrap().clone()
    }

    pub fn insert(&self, job: &Job) -> Job {
        self.rows.lock().unwrap().push(job.clone());
        job.clone()
    }

    /// Stores the job placed on the stream.
    pub fn place(&self, job: &Job, stream: &AgentStream) -> Job {
        self.streams
            .lock()
            .unwrap()
            .insert(job.id().clone(), stream.id.clone());
        self.insert(&stored(job, Some(&stream.agent)))
    }

    pub fn row(&self, id: &JobId) -> Job {
        self.rows()
            .into_iter()
            .find(|j| j.id() == id)
            .expect("the job is stored")
    }

    /// The stream that holds the job.
    pub fn stream_of(&self, id: &JobId) -> Option<StreamId> {
        self.streams.lock().unwrap().get(id).cloned()
    }

    pub fn released(&self) -> Vec<AgentStream> {
        self.released.lock().unwrap().clone()
    }

    /// A concurrent write: the stored job moves to its next version.
    pub fn touch(&self, id: &JobId) {
        let mut rows = self.rows.lock().unwrap();
        if let Some(row) = rows.iter_mut().find(|j| j.id() == id) {
            let agent = row.agent_app_id().cloned();
            *row = stored(row, agent.as_ref());
        }
    }

    fn live(&self, keep: impl Fn(&Job) -> bool) -> Vec<Job> {
        self.rows()
            .into_iter()
            .filter(|j| !j.is_terminal() && keep(j))
            .collect()
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
            .ok_or_else(|| DomainError::not_found("Job", id))
    }
    async fn update(&self, job: &Job) -> DomainResult<Job> {
        let mut rows = self.rows.lock().unwrap();
        let row = rows
            .iter_mut()
            .find(|j| j.id() == job.id())
            .ok_or_else(|| DomainError::not_found("Job", job.id()))?;
        if row.version() != job.version() {
            return Err(DomainError::stale("Job", job.id()));
        }
        let agent = row.agent_app_id().cloned();
        *row = stored(job, agent.as_ref());
        Ok(row.clone())
    }
    async fn delete(&self, job: &Job) -> DomainResult<()> {
        let mut rows = self.rows.lock().unwrap();
        match rows.iter().position(|j| j.id() == job.id()) {
            None => Err(DomainError::not_found("Job", job.id())),
            Some(i) if rows[i].version() != job.version() => {
                Err(DomainError::stale("Job", job.id()))
            }
            Some(i) => {
                rows.remove(i);
                Ok(())
            }
        }
    }
    async fn claim_next(
        &self,
        stream: &AgentStream,
        visible: &Visibility,
    ) -> DomainResult<Option<(Job, ProjectId)>> {
        let mut rows = self.rows.lock().unwrap();
        let busy = rows
            .iter()
            .any(|j| !j.is_terminal() && j.agent_app_id() == Some(&stream.agent));
        if visible.is_empty() || busy {
            return Ok(None);
        }
        let Some(row) = rows
            .iter_mut()
            .find(|j| j.status() == JobStatus::Pending && j.agent_app_id().is_none())
        else {
            return Ok(None);
        };
        *row = stored(row, Some(&stream.agent));
        self.streams
            .lock()
            .unwrap()
            .insert(row.id().clone(), stream.id.clone());
        Ok(Some((row.clone(), ProjectId::new("project"))))
    }
    async fn release(&self, stream: &AgentStream) -> DomainResult<u64> {
        self.released.lock().unwrap().push(stream.clone());
        let mut streams = self.streams.lock().unwrap();
        let mut released = 0;
        for row in self.rows.lock().unwrap().iter_mut() {
            if row.status() == JobStatus::Pending
                && row.agent_app_id() == Some(&stream.agent)
                && streams.get(row.id()) == Some(&stream.id)
            {
                streams.remove(row.id());
                *row = stored(row, None);
                released += 1;
            }
        }
        Ok(released)
    }
    async fn pending_streams(&self) -> DomainResult<Vec<AgentStream>> {
        let streams = self.streams.lock().unwrap();
        let mut held: Vec<AgentStream> = Vec::new();
        for job in self.live(|j| j.status() == JobStatus::Pending) {
            if let (Some(agent), Some(id)) = (job.agent_app_id(), streams.get(job.id())) {
                let stream = AgentStream {
                    agent: agent.clone(),
                    id: id.clone(),
                };
                if !held.contains(&stream) {
                    held.push(stream);
                }
            }
        }
        Ok(held)
    }
    async fn active_jobs(&self, agents: &[AppId]) -> DomainResult<Vec<(AppId, u32)>> {
        let mut counts: Vec<(AppId, u32)> = Vec::new();
        for job in self.live(|_| true) {
            let Some(agent) = job.agent_app_id().filter(|a| agents.contains(a)) else {
                continue;
            };
            match counts.iter_mut().find(|(a, _)| a == agent) {
                Some((_, n)) => *n += 1,
                None => counts.push((agent.clone(), 1)),
            }
        }
        Ok(counts)
    }
    async fn list_live(&self, scope: JobScope<'_>) -> DomainResult<Vec<Job>> {
        Ok(self.live(|j| match scope {
            JobScope::Pipeline(id) => j.pipeline_id() == id,
            JobScope::All | JobScope::Project(_) | JobScope::Organization(_) => true,
        }))
    }
    async fn list_running_on(&self, agent: &AppId) -> DomainResult<Vec<Job>> {
        Ok(self.live(|j| j.status() == JobStatus::Running && j.agent_app_id() == Some(agent)))
    }
    async fn list_stranded(&self, _: DateTime<Utc>) -> DomainResult<Vec<Job>> {
        Ok(self.live(|j| j.status() == JobStatus::Running))
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

/// Records each tail opened and closed and each line published; a subscribe gets no line.
#[derive(Default)]
pub struct StubTails {
    opened: Mutex<Vec<JobId>>,
    closed: Mutex<Vec<JobId>>,
    published: Mutex<Vec<JobLog>>,
}

impl StubTails {
    pub fn opened(&self) -> Vec<JobId> {
        self.opened.lock().unwrap().clone()
    }

    pub fn closed(&self) -> Vec<JobId> {
        self.closed.lock().unwrap().clone()
    }

    pub fn published(&self) -> Vec<JobLog> {
        self.published.lock().unwrap().clone()
    }
}

#[async_trait]
impl JobLogStreamPort for StubTails {
    fn open(&self, job_id: &JobId) {
        self.opened.lock().unwrap().push(job_id.clone());
    }
    fn publish(&self, log: &JobLog) {
        self.published.lock().unwrap().push(log.clone());
    }
    fn close(&self, job_id: &JobId) {
        self.closed.lock().unwrap().push(job_id.clone());
    }
    async fn subscribe(&self, _: &JobId, _: Option<&NodeId>) -> DomainResult<JobLogLiveStream> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

/// What each agent sees; an agent it does not name sees `otherwise`. Records each key asked.
pub struct ScopesByAgent {
    scopes: HashMap<AppId, Visibility>,
    otherwise: Visibility,
    asked: Mutex<Vec<String>>,
}

impl ScopesByAgent {
    pub fn everything() -> Self {
        Self::new(HashMap::new(), Visibility::All)
    }

    pub fn new(scopes: HashMap<AppId, Visibility>, otherwise: Visibility) -> Self {
        Self {
            scopes,
            otherwise,
            asked: Mutex::default(),
        }
    }

    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }
}

#[async_trait]
impl VisibilityResolver for ScopesByAgent {
    async fn visible_scopes(&self, caller: &CallerContext, key: &str) -> DomainResult<Visibility> {
        self.asked.lock().unwrap().push(key.to_string());
        Ok(match caller {
            CallerContext::App(id) => self.scopes.get(id).unwrap_or(&self.otherwise).clone(),
            _ => Visibility::none(),
        })
    }
}

/// A dispatcher on stubs: every agent sees everything and each secret resolves to itself.
pub fn dispatcher(
    registry: Arc<dyn AgentDispatch>,
    jobs: Arc<dyn JobRepository>,
    tails: Arc<StubTails>,
) -> Arc<DispatchUseCases> {
    Arc::new(DispatchUseCases::new(
        registry,
        Arc::new(ScopesByAgent::everything()),
        jobs,
        Arc::new(EchoResolver),
        tails,
    ))
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
        Err(DomainError::not_found("User", id))
    }
    async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
        Ok(Vec::new())
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        Err(DomainError::not_found("User", username))
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        Err(DomainError::not_found("User", email))
    }
    async fn update(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn delete(&self, _: &User) -> DomainResult<()> {
        unreachable!("no user write in this action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        empty_page()
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
        _: &str,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Project>> {
        empty_page()
    }
    async fn find_by_id(&self, id: &ProjectId) -> DomainResult<Project> {
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("Project", id))
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
            Err(DomainError::not_found("Pipeline", id))
        }
    }
    async fn update(&self, _: &Pipeline) -> DomainResult<Pipeline> {
        unreachable!("no pipeline write in this action")
    }
    async fn delete(&self, _: &Pipeline) -> DomainResult<()> {
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

/// Lists its fixed rows and records each write. A create of a grant equal to a row returns
/// that row, as the store does.
#[derive(Default)]
pub struct StubGrants {
    rows: Vec<Grant>,
    created: Mutex<Vec<Grant>>,
    deleted: Mutex<Vec<String>>,
    revoked_all: Mutex<Vec<(Principal, Scope)>>,
}

impl StubGrants {
    pub fn new(rows: Vec<Grant>) -> Self {
        Self {
            rows,
            ..Self::default()
        }
    }

    pub fn created(&self) -> Vec<Grant> {
        self.created.lock().unwrap().clone()
    }

    pub fn deleted(&self) -> Vec<String> {
        self.deleted.lock().unwrap().clone()
    }

    pub fn revoked_all(&self) -> Vec<(Principal, Scope)> {
        self.revoked_all.lock().unwrap().clone()
    }
}

#[async_trait]
impl GrantRepository for StubGrants {
    async fn list_all(&self) -> DomainResult<Vec<Grant>> {
        Ok(self.rows.clone())
    }
    async fn create(&self, grant: &Grant) -> DomainResult<Grant> {
        let stored = self
            .rows
            .iter()
            .find(|g| {
                g.principal == grant.principal && g.role == grant.role && g.scope == grant.scope
            })
            .cloned()
            .unwrap_or_else(|| grant.clone());
        self.created.lock().unwrap().push(stored.clone());
        Ok(stored)
    }
    async fn delete(&self, id: &str) -> DomainResult<()> {
        self.deleted.lock().unwrap().push(id.to_string());
        Ok(())
    }
    async fn revoke_all(&self, principal: &Principal, scope: &Scope) -> DomainResult<u64> {
        self.revoked_all
            .lock()
            .unwrap()
            .push((principal.clone(), scope.clone()));
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
            Err(DomainError::not_found("User", id))
        }
    }
    async fn find_by_ids(&self, _: &[UserId]) -> DomainResult<Vec<User>> {
        Ok(vec![self.0.clone()])
    }
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        if username == self.0.username() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("User", username))
        }
    }
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        if Some(email) == self.0.email() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("User", email))
        }
    }
    async fn update(&self, _: &User) -> DomainResult<User> {
        unreachable!("no user write in this action")
    }
    async fn delete(&self, _: &User) -> DomainResult<()> {
        unreachable!("no user write in this action")
    }
    async fn list_all(&self, _: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
        empty_page()
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
