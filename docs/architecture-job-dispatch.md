# Architecture Job Dispatch — Scylla Core ↔ Agents via Hermes

## Context

Scylla a un **control plane** (scylla-api) avec gRPC, SurrealDB, et un domaine riche (Pipeline DAG avec validation de cycles, Job avec state machine Pending→Running→Completed/Failed/Cancelled/Orphaned, tracking node-level). Un **message broker Hermes** (gRPC, pub/sub, ACK, redelivery, durable store Redb) tourne dans scylla-broker. Un **scylla-agent** vide attend d'être implémenté.

**Objectif** : Designer et implémenter le système de dispatch de jobs — un user lance un pipeline, l'orchestrateur résout le DAG, dispatch les nodes aux agents via Hermes, et gère les résultats/échecs.

## Architecture Globale

```
   scylla-api (control plane)
   ┌─────────────────────────────────────────────┐
   │  gRPC Handlers → JobUseCases                │
   │       │                                      │
   │       v            mpsc channel              │
   │  RunPipeline ──────────────> Orchestrator    │
   │                               │   ↑          │
   │                     publish   │   │ subscribe │
   │                               v   │          │
   │                          HermesClient        │
   │  SurrealDB (Job state)                       │
   └──────────────────────┬───────────────────────┘
                          │ gRPC (Hermes protocol)
                          v
   ┌──────────────────────────────────────────────┐
   │  scylla-broker (Hermes)                      │
   │  BrokerEngine + RedbStore + Redelivery + GC  │
   └──────────────────────┬───────────────────────┘
                          │ gRPC (Hermes protocol)
          ┌───────────────┼───────────────┐
          v               v               v
   scylla-agent-01  scylla-agent-02  scylla-agent-N
   (subscribe, execute commands, publish reports)
```

## Lifecycle d'un Job

1. **User** appelle `RunPipeline(pipeline_id)` via gRPC
2. **Handler** charge le Pipeline, crée un `Job::create_from_pipeline()`, persiste en DB (status=Pending), retourne la réponse immédiatement
3. **Handler** envoie `JobCreated { job_id, pipeline_id }` dans un `mpsc::channel` vers l'Orchestrator
4. **Orchestrator** charge Job+Pipeline, transition Job→Running, construit un `DagExecutionState` en mémoire
5. **Orchestrator** calcule les `ready_nodes()` (dépendances satisfaites), publie un `NodeDispatch` par node ready sur le topic `node.dispatch` via Hermes
6. **Agent** reçoit le `NodeDispatch`, ACK le message (claim), publie `NodeReport::Started`, spawn le process
7. **Agent** stream les logs via `node.logs`, attend la fin du process
8. **Agent** publie `NodeReport::Completed` ou `NodeReport::Failed`
9. **Orchestrator** reçoit le report (via subscribe `node.report`), met à jour le Job en DB, recalcule `ready_nodes()` → dispatch les nodes suivants
10. Quand tous les nodes sont terminaux : `Job.complete()` ou `Job.fail()` selon la policy

## Sequence Diagram

```
User                  API/Handler          Orchestrator          Broker           Agent
 |                     |                     |                   |                |
 |--- RunPipeline ---->|                     |                   |                |
 |                     |-- load Pipeline     |                   |                |
 |                     |-- Job::create_from_pipeline()           |                |
 |                     |-- persist Job (Pending) to SurrealDB    |                |
 |                     |-- send JobCreated via mpsc channel      |                |
 |<-- JobResponse -----|                     |                   |                |
 |                     |                     |                   |                |
 |                     |               recv JobCreated           |                |
 |                     |               load Pipeline + Job       |                |
 |                     |               build DagExecutionState   |                |
 |                     |               Job.start() -> Running    |                |
 |                     |               persist to SurrealDB      |                |
 |                     |               ready_nodes() -> [a]      |                |
 |                     |                     |                   |                |
 |                     |                     |-- NodeDispatch -->|                |
 |                     |                     |   {node:"a"}      |-- deliver ---->|
 |                     |                     |                   |                |
 |                     |                     |                   |<--- ACK -------|
 |                     |                     |                   |                |
 |                     |                     |<-- NodeReport ----|<-- publish ----|
 |                     |                     |   Started(a)      |                |
 |                     |                     |                   |                |
 |                     |                     |<-- NodeReport ----|<-- publish ----|
 |                     |                     |   Completed(a)    |                |
 |                     |                     |                   |                |
 |                     |               apply_node_finished(a)    |                |
 |                     |               persist to SurrealDB      |                |
 |                     |               ready_nodes() -> [b, c]   |                |
 |                     |                     |                   |                |
 |                     |                     |-- NodeDispatch -->|-- deliver ---->|
 |                     |                     |   {node:"b"}      |                |
 |                     |                     |-- NodeDispatch -->|-- deliver ---->|
 |                     |                     |   {node:"c"}      |  (other agent) |
 |                     |                     |                   |                |
 |                     |               all nodes completed       |                |
 |                     |               Job.complete()            |                |
```

## Nouveau crate : `scylla-dispatch`

```
crates/scylla-dispatch/
  src/
    lib.rs               # re-exports
    messages.rs          # NodeDispatch, NodeReport, AgentHeartbeat, Envelope<T>
    dag_state.rs         # DagExecutionState (in-memory DAG tracker)
    orchestrator.rs      # Orchestrator loop (level-triggered reconciliation)
    publisher.rs         # Typed HermesPublisher wrapper
    subscriber.rs        # HermesSubscriber → OrchestratorEvent
    config.rs            # OrchestratorConfig
    error.rs             # DispatchError
```

### DagExecutionState

Tracker in-memory du DAG d'un job en cours. Rebuildable from `(Pipeline, Job)` pour crash recovery.

```rust
pub struct DagExecutionState {
    pub job_id: JobId,
    pub pipeline_id: PipelineId,
    pub adjacency: HashMap<NodeId, Vec<NodeId>>,      // node → who depends on it
    pub reverse_adj: HashMap<NodeId, Vec<NodeId>>,     // node → its deps
    pub node_states: HashMap<NodeId, NodeState>,
    pub node_commands: HashMap<NodeId, (String, Vec<String>)>,
    pub agent_assignments: HashMap<NodeId, String>,
    pub last_activity: HashMap<NodeId, DateTime<Utc>>,
}
```

Méthodes clés :
- `from_pipeline_and_job()` — reconstruit depuis DB (crash recovery)
- `ready_nodes()` — nodes Pending dont tous les deps sont Completed
- `is_finished()` / `has_failure()`
- `mark_running()` / `mark_finished()`

### Orchestrator (pattern Kubernetes controller — level-triggered)

À chaque événement (nouveau job, report d'un agent, tick timer), l'orchestrateur réexamine l'état complet et prend toutes les actions nécessaires. Ce pattern est naturellement **idempotent** et **recoverable**.

```rust
pub struct Orchestrator {
    event_rx: mpsc::Receiver<OrchestratorEvent>,
    active_jobs: HashMap<JobId, DagExecutionState>,
    publisher: HermesPublisher,
    job_repo: Arc<dyn JobRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    config: OrchestratorConfig,
    known_agents: HashMap<String, AgentInfo>,
}

pub enum OrchestratorEvent {
    JobCreated { job_id, pipeline_id },
    NodeStarted { job_id, node_id, agent_id },
    NodeFinished { job_id, node_id, state, exit_code, error },
    AgentHeartbeat { agent_id, info },
    CancelJob { job_id },
    Tick,   // generated from recv timeout
}
```

Boucle principale : `loop { recv event with timeout → handle_event }`. Le timeout génère un `Tick` pour la réconciliation périodique (timeout detection, re-dispatch).

**Crash recovery** : au démarrage, `recover_active_jobs()` charge les jobs Running depuis SurrealDB, reconstruit les `DagExecutionState`, re-subscribe et reconcile.

**Concurrence DAG** : les nodes indépendants sont dispatchés en parallèle (ex: `a → [b, c] → d` : après `a`, `b` et `c` sont dispatchés simultanément).

### OrchestratorConfig

```rust
pub struct OrchestratorConfig {
    pub tick_interval_secs: u64,       // default 10
    pub node_timeout_secs: u64,        // default 3600
    pub agent_timeout_secs: u64,       // default 45 (3x heartbeat)
    pub failure_policy: FailurePolicy, // default FailFast
}
```

## Messages Hermes

| Topic | Publisher | Subscriber | Payload |
|---|---|---|---|
| `node.dispatch` | Orchestrator | Agents | `NodeDispatch` |
| `node.report` | Agents | Orchestrator | `NodeReport` |
| `node.logs` | Agents | Orchestrator (opt) | `NodeLogChunk` |
| `agent.heartbeat` | Agents | Orchestrator | `AgentHeartbeat` |
| `job.cancel` | Orchestrator | Agents | `JobCancelCommand` |

### Schemas

```rust
// ── Envelope (wraps all messages) ───────────────────────────
pub struct Envelope<T: Serialize> {
    pub version: u32,           // schema version, start at 1
    pub message_id: String,     // ULID for idempotency
    pub timestamp: DateTime<Utc>,
    pub payload: T,
}

// ── Orchestrator → Agent ────────────────────────────────────
pub struct NodeDispatch {
    pub job_id: String,
    pub pipeline_id: String,
    pub node_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub timeout_secs: Option<u64>,
    pub attempt: u32,           // idempotency key: (job_id, node_id, attempt)
}

pub struct JobCancelCommand {
    pub job_id: String,
}

// ── Agent → Orchestrator ────────────────────────────────────
pub enum NodeReportKind {
    Started,
    Completed { exit_code: i32 },
    Failed { exit_code: Option<i32>, error: String },
}

pub struct NodeReport {
    pub job_id: String,
    pub node_id: String,
    pub agent_id: String,
    pub kind: NodeReportKind,
    pub timestamp: DateTime<Utc>,
}

pub struct NodeLogChunk {
    pub job_id: String,
    pub node_id: String,
    pub agent_id: String,
    pub data: Vec<u8>,
    pub is_stderr: bool,
    pub sequence: u64,          // monotonic, for ordering
    pub timestamp: DateTime<Utc>,
}

// ── Agent → Orchestrator (liveness) ─────────────────────────
pub struct AgentHeartbeat {
    pub agent_id: String,
    pub labels: Vec<String>,
    pub running_nodes: Vec<RunningNodeInfo>,
    pub capacity: u32,
    pub available: u32,
}

pub struct AgentRegistration {
    pub agent_id: String,
    pub labels: Vec<String>,
    pub capacity: u32,
    pub version: String,
}
```

### Stratégie ACK

L'agent ACK le message Hermes au moment du **claim** (enregistré en in-flight set, avant le spawn du process). Pas à la fin de l'exécution — sinon Hermes redeliver pendant l'exécution et crée des doublons.

| Strategy | Pros | Cons |
|----------|------|------|
| ACK on receipt | Fastest | Agent crash before spawn = lost work |
| **ACK on claim (chosen)** | Message cleared after agent commits | Small window between claim and spawn |
| ACK on completion | Guaranteed delivery | Redelivery during execution = duplicates |

## Design de l'Agent

```rust
pub struct Agent {
    config: AgentConfig,
    hermes: HermesClient,
    running: Arc<Mutex<HashMap<(String, String), RunningNode>>>,
    semaphore: Arc<Semaphore>,  // max_concurrent
    cancel_tokens: Arc<Mutex<HashMap<String, Vec<CancellationToken>>>>,
}

pub struct AgentConfig {
    pub broker_url: String,
    pub agent_id: String,              // ULID or hostname-based
    pub labels: Vec<String>,           // e.g., ["linux", "docker", "gpu"]
    pub max_concurrent: u32,           // default 4
    pub heartbeat_interval_secs: u64,  // default 15
    pub work_dir: PathBuf,
    pub node_timeout_secs: u64,        // default 3600
}
```

### Flow d'exécution d'un node

1. `semaphore.acquire()` — backpressure naturelle (bloque si capacité atteinte)
2. Check idempotence : skip si `(job_id, node_id)` déjà en cours
3. Publish `NodeReport::Started`
4. Créer work_dir isolé : `{work_dir}/{job_id}/{node_id}/`
5. `tokio::process::Command::new(cmd).args(args).kill_on_drop(true).spawn()`
6. Stream stdout/stderr → `NodeLogChunk` sur `node.logs`
7. `tokio::select!` entre : `child.wait()`, `cancel_token.cancelled()`, `timeout`
8. Publish `NodeReport::Completed` ou `Failed` (avec exit code et error message)
9. Cleanup in-flight set, release semaphore permit

### Heartbeat

Toutes les 15s, publie `AgentHeartbeat { agent_id, labels, running_nodes, capacity, available }`.

### Agents stateless

Pas de registry central. L'orchestrateur découvre les agents via heartbeats. Scaling = lancer plus d'agents. Un agent qui meurt disparaît simplement après 45s de silence.

## Gestion des Échecs

| Échec | Détection | Réaction |
|---|---|---|
| Node exit non-zero | Agent lit exit code | Report Failed → FailFast (cancel tout) |
| Node timeout | Agent timer | Kill process, report Failed |
| Agent meurt | Heartbeat stop 45s | Orchestrator marque node Failed |
| Broker restart | TCP reconnect auto | Redelivery des messages non-ACK |
| API/Orchestrator crash | Process restart | `recover_active_jobs()` depuis DB |
| Pipeline supprimé pendant job | NotFound au load | Job → Orphaned |

### FailurePolicy

Phase 1 implémente **FailFast** seulement :

```rust
pub enum FailurePolicy {
    /// Fail the entire job immediately. Cancel pending/running nodes.
    FailFast,
    /// Only cancel nodes that transitively depend on the failed node.
    FailBranch,
    /// Retry the failed node up to N times before giving up.
    Retry { max_attempts: u32, backoff_secs: u64 },
}
```

**FailFast sequence** :
1. Node reports Failed
2. Orchestrator updates node in DB
3. Orchestrator cancels all Pending nodes (set to Cancelled in DB)
4. Orchestrator publishes `JobCancelCommand` to `job.cancel` topic
5. Agents running nodes for this job kill their processes and report Failed
6. Once all nodes are terminal, Job transitions to Failed

### Idempotence

| Operation | Idempotency Key | Duplicate Behavior |
|-----------|----------------|--------------------|
| NodeDispatch | `(job_id, node_id, attempt)` | Agent checks in-flight set, ignores duplicate |
| NodeReport::Started | `(job_id, node_id)` | Orchestrator accepts first, ignores second |
| NodeReport::Completed/Failed | `(job_id, node_id)` | `apply_node_finished()` rejects if node not Running |
| Job state transitions | `JobStatus::transition_to()` | Returns error on invalid transition |

### Race Conditions

| Race Condition | Mitigation |
|----------------|------------|
| Double dispatch (crash between in-memory mark and DB persist) | Agent idempotency: ignores if already running `(job_id, node_id)` |
| Double completion (broker redelivers NodeReport) | `apply_node_finished()` rejects if node not Running |
| Cancel during dispatch | Cancel sets Job to Cancelled in DB first. Agent receives `job.cancel` and kills process |
| Agent claims then dies | Heartbeat timeout (45s). Orchestrator marks node Failed |
| Pipeline mutated during job | Store NodeSpec in Job (snapshot at creation) |

## Modifications au Domaine (scylla-core)

### Job entity — ajouts

```rust
// Nouveaux champs sur JobNode
pub struct JobNode {
    node_id: NodeId,
    state: NodeState,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    agent_id: Option<String>,       // NEW
    exit_code: Option<i32>,         // NEW
    error_message: Option<String>,  // NEW
    attempt: u32,                   // NEW
}

// Nouvelles méthodes sur Job
impl Job {
    pub fn all_nodes_terminal(&self) -> bool;
    pub fn all_nodes_completed(&self) -> bool;
    pub fn cancel_pending_nodes(&mut self);
}
```

### JobRepository — ajout

```rust
async fn find_by_status(&self, status: JobStatus) -> DomainResult<Vec<Job>>;
```

### Point critique : Snapshot du Pipeline dans le Job

Le Job actuel ne stocke que `pipeline_id` + node IDs. Si le pipeline est modifié entre la création du job et l'exécution, les commandes seront fausses.

**Solution** : stocker les `NodeSpec` (node_id, deps, command, args) dans le Job au moment de la création :

```rust
pub struct NodeSpec {
    pub node_id: NodeId,
    pub deps: Vec<NodeId>,
    pub command: String,
    pub args: Vec<String>,
}

pub struct Job {
    // ... existing fields ...
    node_specs: Vec<NodeSpec>,  // immutable execution plan
}
```

Le Job devient self-contained — l'orchestrateur n'a plus besoin du Pipeline pendant l'exécution.

### Proto (job.proto) — nouveaux RPCs

```protobuf
service JobService {
  // ... existing RPCs ...
  rpc RunPipeline(RunPipelineRequest) returns (JobResponse);
  rpc CancelJob(CancelJobRequest) returns (JobResponse);
  rpc GetJobLogs(GetJobLogsRequest) returns (stream JobLogChunk);
}

message RunPipelineRequest {
  string pipeline_id = 1;
  map<string, string> env_vars = 2;
}

message CancelJobRequest {
  string job_id = 1;
}

message GetJobLogsRequest {
  string job_id = 1;
  string node_id = 2;
}

message JobLogChunk {
  string node_id = 1;
  bytes data = 2;
  bool is_stderr = 3;
  string timestamp = 4;
}
```

## Intégration dans scylla-api

Dans `startup.rs` :
```rust
let (orch_tx, orch_rx) = mpsc::channel::<OrchestratorEvent>(256);

let orchestrator = Orchestrator::new(orch_rx, publisher, job_repo, pipeline_repo, config);
let report_subscriber = ReportSubscriber::new(subscriber, orch_tx.clone());

tokio::spawn(orchestrator.run());
tokio::spawn(report_subscriber.run());
```

`Services` gagne un `orchestrator_tx: mpsc::Sender<OrchestratorEvent>`.
`JobHandler::run_pipeline()` crée le job, persist, send `JobCreated` dans le channel.

## Sécurité

### Agent Authentication
- Phase 1 : shared secret `SCYLLA_AGENT_TOKEN` en env var, envoyé comme gRPC metadata
- Phase 2 : mTLS (cert client par agent)
- Phase 3 : JWT dynamique (agent request registration token from API)

### Command Sandboxing
- Phase 1 : `kill_on_drop(true)`, work_dir isolé par node, user OS dédié
- Phase 2 : exécution dans un container (podman/docker)
- Phase 3 : microVM (Firecracker) pour high-security

## Scaling

### Phase 1 Target
10 concurrent jobs, 100 nodes in flight, 5 agents. Single orchestrator.

### Bottleneck Analysis
| Bottleneck | Threshold | Mitigation |
|------------|-----------|------------|
| Single orchestrator | ~500 active jobs | Extract to separate service, partition by job_id hash |
| SurrealDB write rate | ~1000 updates/sec | Batch node state updates into single Job write |
| Hermes broker memory | ~10K unACKed messages | Scale broker or partition topics |
| Log streaming volume | ~100 nodes at 1MB/s | Write logs to object storage from agent directly |

### Agent Labels et Affinity (Phase 3)
```rust
// NodeDispatch gains:
pub required_labels: Vec<String>,

// Orchestrator selects least-loaded matching agent
```

## Phases d'implémentation

### Phase 1 : MVP Dispatch
- [ ] Créer `scylla-dispatch` avec messages, dag_state, orchestrator
- [ ] Ajouter `NodeSpec` au Job, modifier `create_from_pipeline()`
- [ ] Ajouter `find_by_status()` au JobRepository + impl SurrealDB
- [ ] Ajouter RPCs `RunPipeline`, `CancelJob` au proto
- [ ] Implémenter le handler `run_pipeline` + wiring orchestrator dans startup
- [ ] Implémenter l'agent basique : subscribe, execute, report
- [ ] Tests unitaires dag_state + orchestrator (avec stubs)

### Phase 2 : Résilience
- [ ] `recover_active_jobs()` au démarrage
- [ ] Heartbeat agent + timeout detection orchestrator
- [ ] Cancel flow complet (RPC → topic → agent kill)
- [ ] Idempotence agent + orchestrator
- [ ] Log streaming `NodeLogChunk` + `GetJobLogs` RPC

### Phase 3 : Maturité
- [ ] Agent labels + affinity
- [ ] Retry policy per-node
- [ ] FailBranch policy
- [ ] Auth agent (shared token)
- [ ] Métriques (job duration, queue depth, agent utilization)

## Fichiers à créer/modifier

### Nouveaux
| Fichier | Contenu |
|---|---|
| `crates/scylla-dispatch/Cargo.toml` | Dépendances : scylla-core, hermes-broker-client/core, serde, tokio, chrono, tracing |
| `crates/scylla-dispatch/src/lib.rs` | Re-exports |
| `crates/scylla-dispatch/src/messages.rs` | NodeDispatch, NodeReport, AgentHeartbeat, Envelope |
| `crates/scylla-dispatch/src/dag_state.rs` | DagExecutionState |
| `crates/scylla-dispatch/src/orchestrator.rs` | Orchestrator loop |
| `crates/scylla-dispatch/src/publisher.rs` | HermesPublisher |
| `crates/scylla-dispatch/src/subscriber.rs` | HermesSubscriber → events |
| `crates/scylla-dispatch/src/config.rs` | OrchestratorConfig |
| `crates/scylla-dispatch/src/error.rs` | DispatchError |

### Modifiés
| Fichier | Modification |
|---|---|
| `Cargo.toml` (workspace) | Ajouter scylla-dispatch aux members |
| `crates/scylla-core/src/domain/entities/job.rs` | NodeSpec, JobNode étendu, nouvelles méthodes |
| `crates/scylla-core/src/application/ports/repositories/job_repo.rs` | `find_by_status()` |
| `crates/scylla-core/src/infrastructure/persistence/surrealdb/job_repository.rs` | Impl `find_by_status()` |
| `libs/protocol/proto/job.proto` | RunPipeline, CancelJob, GetJobLogs |
| `crates/scylla-api/src/startup.rs` | Wire orchestrator channel + spawn |
| `crates/scylla-api/src/grpc/handlers/job_handler.rs` | run_pipeline, cancel_job |
| `crates/scylla-agent/src/main.rs` | Implémentation complète agent |
| `crates/scylla-agent/Cargo.toml` | Ajouter scylla-dispatch, tokio-util, tracing, chrono |

## Vérification

1. `cargo build --workspace` — compile
2. `cargo test -p scylla-dispatch` — tests unitaires dag_state + orchestrator
3. Test manuel : lancer broker + API + agent, `grpcurl RunPipeline`, vérifier que les nodes s'exécutent en ordre DAG
4. Test échec : kill un agent mid-execution, vérifier timeout + job Failed
5. Test crash recovery : kill l'API mid-job, restart, vérifier reprise
