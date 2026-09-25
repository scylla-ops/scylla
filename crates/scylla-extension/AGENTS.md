# scylla-extension: the action pipeline

Every write in Scylla is a command that moves through three typed stages,
every read is a query that moves through two, and an extension attaches to
each stage. This crate holds the pipeline. It depends on `scylla-domain` and
nothing else: no access model, no database, no gRPC, no Cedar. A hook sees
`CallerContext`, `Access` (with its `Permission` values) and `DomainError`,
never a repository or a wire type. An Enterprise build compiles it from a
pinned tag.

Run `cargo test -p scylla-extension` for the in-crate checks. They drive the
engine with a self-contained aggregate (`src/tests.rs`), so a change to the
pipeline is proven here before it reaches a use case.

## Layout

```
src/
  action/        the event
    id.rs          ActionId: one per run
    command.rs     Describe: access(); Command: Staged, Committed; Query
    value.rs       Draft<T> (not stored), Deleted<T> (tombstone)
    envelope.rs    Envelope<C>: id, at, caller, access, command; one Arc
    phase.rs       Requested, Authorized, Prepared, Committed, Fetched
    erased.rs      Action: the erased view of a phase; Phase: its envelope
  stage.rs       StageKind, Stage, Authorize/Prepare/Persist, Fetch, Run<S>
  authz.rs       Access, Granted, Authorizer, AuthorizeStage
  hooks/         the extension seam
    position.rs    Policy, Gate, Around, Wrap, Listener, Observer
    next.rs        Next: rest of a typed chain; Proceed, Done: the erased one
    registry.rs    Hooks: registration and run()
    extension.rs   Extension: register(self: &Arc<Self>, &mut Hooks)
  path.rs        Path<K, R>: the chain after Authorize (Write, Read)
  actions.rs     Actions: run(runner, caller, action)
  tests.rs       the checks, on a fake aggregate
```

The core side lives in `scylla-core`: `application/actions.rs` adapts
`PermissionService` to `Authorizer`, `application/project/` is the reference
use case (`commands.rs`, `queries.rs`, the ports in `mod.rs`), and
`grpc/handlers/project_handler.rs` is the reference adapter.

## The event

One action is one event. It is born in `Requested<C>`, where `C` is the
command or the query. `Requested<C>` holds an `Arc<Envelope<C>>`: the action
id, the time, the caller, the access rule and the complete command. Every
later phase keeps the same envelope. All phases deref to it, so `caller()`,
`access()`, `command()` and `id()` are available at each step. No phase has a
public constructor. The only way to get a later phase is a transition method
on the earlier phase, and each transition consumes its input.

| phase           | new data                       | who produces it       |
|-----------------|--------------------------------|-----------------------|
| `Requested<C>`  | envelope                       | `Actions`             |
| `Authorized<C>` | nothing; `Granted` is consumed | the authorize stage   |
| `Prepared<C>`   | `C::Staged`                    | the use case          |
| `Committed<C>`  | `C::Committed`                 | the store             |
| `Fetched<Q>`    | `Q::Output`                    | the use case          |

`Granted` is a token with a private constructor and no data. Only
`AuthorizeStage` makes one, and `Requested::authorized` consumes it, so an
`Authorized<C>` is proof that the access check ran.

A command or a query declares its access rule through `Describe`, and its
payload types through `Command` or `Query`. Nothing else: which path it takes
follows from the trait.

`Access` has four values. The authorize stage applies them as follows:

| access                    | who passes                        | calls the `Authorizer`   |
|---------------------------|-----------------------------------|--------------------------|
| `Public`                  | every caller, `Anonymous` too     | no                       |
| `Authenticated`           | every caller except `Anonymous`   | no                       |
| `Requires(p)`             | a caller that has `p`             | one time, for `p`        |
| `RequiresAll(vec![p, q])` | a caller that has `p` and `q`     | for each, in order       |

`Authenticated` refuses `Anonymous` with `Forbidden`, the same error as the
Cedar check. For `RequiresAll`, the first refusal stops the check, and the
next permissions are not asked. Use `RequiresAll` when the action must refuse
on two permissions, for example a trigger update, which needs
`manageTriggers` and `runPipeline`. `Access::permissions()` gives the
permissions as a slice: empty for `Public` and `Authenticated`. For each
access, the hooks run in the same way: `Public` and `Authenticated` also go
through `Authorize`, and a `Granted` is minted for them without the
`Authorizer`.

```rust
pub struct CreateProject { pub organization_id: OrganizationId, pub name: ProjectName, ... }

impl Describe for CreateProject {
    fn access(&self) -> Access {
        Access::Requires(Permission::CreateProject(self.organization_id.clone()))
    }
}

impl Command for CreateProject {
    type Staged = Draft<NewProject>;
    type Committed = Project;
}

pub struct GetProject { pub id: ProjectId }

impl Describe for GetProject {
    fn access(&self) -> Access {
        Access::Requires(Permission::ReadProject(self.id.clone()))
    }
}

impl Query for GetProject {
    type Output = Project;
}
```

How the compiler picks the path (`path.rs`): coherence forbids "every
`Command`" and "every `Query`" as two blanket impls of one trait, because
nothing stops a type from being both. So the trait takes a marker parameter,
`Path<K, R>`, with one impl for `K = Write` over every `Command` and one for
`K = Read` over every `Query`. At a call `actions.run(runner, caller, action)`
the compiler tries both impls, keeps the one whose bounds hold, and infers `K`
from it. A wrong or missing `Run` impl on the runner is a compile error at the
call site, worded by `#[diagnostic::on_unimplemented]` on `Path`.

A query is not a command with an empty write. `Committed<T>` means "this is in
the store"; a read that went through a fake `Persist` would make that word
false. So a query has its own second stage and its own output phase, and
shares everything else: the envelope, the authorize stage, the hooks.

A `Draft<T>` is a value that is not in the store. A `Project` that comes out of
`Committed<CreateProject>` is in the store. For `DeleteProject`, `Staged` is
the loaded `Project` and `Committed` is `Deleted<Project>`, a tombstone with
the last state. The type tells you if the thing exists.

`Prepared<C>` has one way out: `commit(async |staged| ...)`. The closure gets
the staged value by move and must return a `C::Committed`. The store writes
inside the closure. A closure that does not write is a simulation, which is
what a dry-run `Wrap` does.

## The stages

Four stages, each a type with an `In` and an `Out` phase. A command takes the
first three, a query takes the first and the last:

- `Authorize<C>`: `Requested<C>` to `Authorized<C>`. `AuthorizeStage` runs it
  for every command and query; it applies the `Access` of the action, asks the
  `Authorizer` (in the core, the Cedar `PermissionService`) for each required
  permission and mints the `Granted`.
- `Prepare<C>`: `Authorized<C>` to `Prepared<C>`. The use case runs it. It
  builds the draft or loads the target. It reads, it does not write.
- `Persist<C>`: `Prepared<C>` to `Committed<C>`. The use case runs it too,
  against its repository port, by calling `commit` and writing inside the
  closure.
- `Fetch<Q>`: `Authorized<Q>` to `Fetched<Q>`. The use case runs it: it reads
  through the ports and returns the output.

`Run<S>` is the trait a stage implementation satisfies:
`async fn run(&self, input: S::In) -> DomainResult<S::Out>`. The signature is
fixed by `S`; the compiler, not the author, decides the phases.

`Actions::run(runner, caller, action)` is the one entry point. It mints the
envelope, opens the span and runs `Authorize`; then `Path::run` takes over
(`path.rs`): a `Command` runs prepare and persist and returns `C::Committed`,
a `Query` runs fetch and returns `Q::Output`. The `Path` impls carry the bound
the runner must satisfy, so a missing `Run` impl is a compile error at the
call site. The runner is the use case struct: one object that implements
the `Run` traits for its commands and queries. There is one `Actions` per
server, built in `init_services` with the permission service and the hooks,
and every action runs in one tracing span with its kind, id, caller, access
and resource; use cases carry no `#[instrument]` of their own. The `access`
field is `public`, `authenticated` or the permission keys joined by `+`; the
`resource` field is there only when a permission is required.

The access check is a stage and not a hook for two reasons. It runs with
zero hooks registered, because `Actions::run` calls it in code; a hook is
something a binary may or may not register. And it has an output,
`Authorized<C>`, that a `Gate` cannot produce: the use case's
`Run<Prepare<C>>` takes `Authorized<C>`, so its signature is the proof that
the check ran.

### The use case and the adapter

A use case struct (`ProjectUseCases`) holds the ports and implements the `Run`
traits. Each port is an `Arc<dyn Port>`, thus the struct, its `Run` impls and
its handler have no type parameters. It has no method of its own, no `Actions`
field, no access code and no hook code. The adapter (a gRPC handler) holds
`Arc<Actions>` and `Arc<ProjectUseCases>`, and each RPC is one call and its
response:

```rust
async fn create_project(&self, request: Request<CreateProjectRequest>)
    -> Result<Response<CreateProjectResponse>, Status> {
    let project = run(&self.actions, &*self.projects, request).await?;
    Ok(Response::new(CreateProjectResponse { project: Some(project_to_proto(&project)) }))
}

async fn list_projects(&self, request: Request<ListProjectsRequest>)
    -> Result<Response<ListProjectsResponse>, Status> {
    let page = run(&self.actions, &*self.projects, request).await?;
    Ok(Response::new(page.into()))
}
```

`grpc::adapter::run` (in `scylla-core`) takes the caller from the
interceptor, turns the request into its command or query through
`grpc::convert::Parse`, runs the engine and maps a `DomainError` to a `Status`.
A service without the interceptor (sign-in, signup, OAuth, the app token
exchange, the invitation accept) uses `grpc::adapter::run_public`: the same
steps, with `Anonymous` as the caller, so only a `Public` action passes. An
HTTP route uses `rest::adapter::run_public`: the route builds the command from
the path, the headers and the body, and maps the `DomainError` to its own
status.

The request types implement `Parse` in the aggregate's mapper
(`grpc/mappers/project_mapper.rs`), built from the small converters in
`grpc::convert`: `id` for a required id wrapper, `valid` for a domain value
built from a wire string, `proto_to_domain_pagination` for a page. When each
field is a required id, the page or a field copied as it is, the mapper uses
`parse!` (`grpc/mappers/macros.rs`) and does not write the impl. The same
mapper turns a page into its response with `From`: `page_response!` when the
response is only the mapped items and the page metadata. An impl with other
logic stays written by hand. A handler never reads a request field itself.

### Adding a command or a query

A use case is one directory: `mod.rs` holds the struct with its ports,
`repository.rs` the port, `commands.rs` the writes, `queries.rs` the reads,
`tests.rs` the checks. Inside `commands.rs`, one block per command in the
order it runs: the struct with public fields, `impl Describe` with the
access, `impl Command` with the two payload types, `impl Run<Prepare<C>>`
(read through the port, build the domain value, `input.prepared(staged)`),
`impl Run<Persist<C>>`
(`input.commit(async |staged| self.repo.write(&staged).await).await`).
Inside `queries.rs`, one block per query: the struct, `impl Describe`,
`impl Query` with the output type, `impl Run<Fetch<Q>>` (read,
`input.fetched(output)`). A new action is one block in the right file, and
the outline of the file is the list of actions. A helper method of the use
case goes in `mod.rs`, not between two blocks.

`Staged` is a `Draft<T>` when `Prepare` builds or changes a domain value that
is not written yet: a new row, a changed row, a list of changed rows, or a
tuple of them. It is the loaded value for a delete, and a bare type only for
a parameter that is not a domain value (a count, a time, an id).

An aggregate with a second runner puts it in a subdirectory with the same
files: `mod.rs`, `commands.rs` (or `queries.rs`), `tests.rs`. For example
`app/token/`, `job/log/`, `trigger/fire/`, `trigger/webhook/` and
`agent/dispatch_use_case/`. A driver that holds `Actions` (a reaper, a
scheduler, a sweeper) has its own file, and its tests check only the driver;
the tests of the actions it sends are in the `tests.rs` of the use case.

A command derives `Debug` when each secret field is a redacting type
(`Password`, `AppSecret`): their `Debug` shows `[REDACTED]`. A command that
holds a secret as a raw `String` or as bytes has no `Debug`: `RevokeToken`,
`ValidateToken`, `OAuthCallback`, `CreateSecret`, `IngestWebhook` and
`AcceptInvitation` (its `token`).

A `tests.rs` gets its engine from
`test_support::authz::actions(permissions)`: the `PermissionAuthorizer` and
no hooks. Use `actions_with(permissions, hooks)` when the test registers
hooks. The stubs that more than one use case needs are in
`test_support::stubs` (compiled for tests only): `CountingPolicy`,
`StubHash`, `StubRegistry`, `StubRoles`, `StubGrants`, `StubJobs`,
`StubSessions`, `StubSignups`, `NoUsers`, `OneUser`, `OneProject`,
`OnePipeline`, `EchoResolver`, `empty_page` and `alice`. `StubRegistry`
fails a test that dispatches a job; `StubRegistry::accepting()` records each
dispatch with its payload. Keep a stub in the `tests.rs` of the use case when
its behavior is specific to that use case, and give it a name that tells what
it does (`RowJobs`, `CountingRoles`), not the name of a shared stub.

A permission check that never refuses is not a gate. `ListOrganizationProjects`
asks a second time for `ListProjectsByOrganization` only to choose between
every project of the organization and the ones the caller's grants reach; that
decision lives in the `Fetch` runner, next to the read it scopes.

## The hook positions

`Hooks::run` surrounds each stage, `Fetch` included, with six positions. They
run in this order:

```
Policy -> Gate<S> -> Around [ Wrap<S> [ Run ] ] -> Listener<S> -> Observer
 veto      veto      control   control            side effect     record
 every     one       every     one                one             every
```

Three of them are erased: `Policy`, `Around` and `Observer` are registered on
a `StageKind` and fire for every command that has a stage of that kind, also
for commands that do not exist yet. They receive `&dyn Action`: the action id,
`at()`, the caller, the `Access`, and `downcast_ref` to the typed phase.
Three of them are typed: `Gate<S>`, `Wrap<S>` and `Listener<S>` are registered
on one stage of one command, for example `Persist<DeleteProject>`, and receive
the exact phase type.

The rule to choose a position: decide first if the hook must be able to stop
the action (`Policy`, `Gate`), control how the work runs (`Around`, `Wrap`),
or only see the result (`Listener`, `Observer`). Then decide if the rule is
about one command (typed) or about a class of actions (erased). Then pick the
stage whose input or output carries the data the hook needs.

### Policy

An erased rule that the edition applies to a class of actions before a stage
runs. Signature: `enforce(&self, stage: StageKind, action: &dyn Action) ->
DomainResult<()>`.

Use it for: quotas, plan limits, rate limits, maintenance mode, feature
flags, kill switches. The decision reads `action.access().permissions()`,
`action.caller()`, and state the extension owns. Do not match on
`Access::Requires(..)`: that pattern does not see a permission inside
`RequiresAll`. A quota returns
`DomainError::quota_exceeded(..)`, which maps to `RESOURCE_EXHAUSTED`.

Do not use it for: a rule that needs one command's fields (use a `Gate`), a
side effect (use an `Observer`), anything that writes.

Which `StageKind`:

- `Authorize`: runs before the access check. Only for rules that must
  apply to unauthorized callers too, for example a rate limit or a
  maintenance mode. A quota here leaks quota state to callers who have no
  right to create.
- `Prepare`: runs after the access check and before the use case. The
  default position for a policy.
- `Persist`: runs after the use case and before the store. Use it when the
  rule needs the staged value, reached through `downcast_ref`.
- `Fetch`: runs after the access check and before the read. A rate limit
  on reads goes here, or on `Authorize` to cover reads and writes at once.

A `Policy` must be idempotent and must not write.

### Gate

A typed rule about one stage of one command. Signature:
`check(&self, input: &S::In) -> DomainResult<()>`.

Use it for: a rule that reads the fields of one command or one staged value.
"A project whose name starts with `prod-` cannot be deleted" reads
`Prepared<DeleteProject>::staged()`, so it is a `Gate<Persist<DeleteProject>>`.
"A trigger with a schedule needs the team plan" reads the command, so it is a
`Gate<Prepare<CreateTrigger>>`. A refusal is `DomainError::business_rule(..)`.

Do not use it for: a rule that applies to several commands (use a `Policy`,
or one `Gate` per command if the rules differ), a side effect, a change to
the input (it is a shared reference; a `Wrap` changes inputs).

Pick the stage by the data: the command alone is there at `Authorize`, the
staged value at `Persist`. A gate on `Persist` decides on the staged value as
read in `Prepare`; the version check makes sure the write goes through only if
the row is still in that state.

### Around

An erased hook that controls how a stage runs, for every command. Signature:
`around(&self, stage: StageKind, next: Proceed<'_>) -> DomainResult<Done>`.

Use it for: timing, a tracing span, a metric of failures, anything that must
surround every action of a stage kind. `next.action()` gives the input as
`&dyn Action`; `next.run().await` runs the rest of the chain and returns a
`Done`. The hook must return that `Done`; it cannot build one, so it cannot
skip the stage.

Do not use it for: anything that reads or changes the typed input or output
(use a `Wrap`), a decision (use a `Policy`; an `Around` that returns `Err`
without calling `next` is a veto in the wrong place).

Several `Around` on the same kind nest. The first registered is the
outermost. Every `Around` runs outside every `Wrap`.

### Wrap

A typed hook that controls how the stage's work runs for one command.
Signature:
`wrap(&self, input: S::In, next: Next<'_, S>) -> DomainResult<S::Out>`.

Use it for: a cache in front of a read, a dry-run mode, anything that needs
the typed input or output. It calls `next.run(input)` exactly once in normal
operation. A read cache is a `Wrap<Fetch<GetProject>>` that answers with
`input.fetched(cached)` on a hit and stores `fetched.output()` on a miss. A dry
run is a `Wrap<Persist<C>>` that never calls `next` and builds its output with
`input.commit(async |draft| Ok(draft.into_inner()))`.

Do not use it for: a business rule (a `Wrap` that returns `Err` to refuse is
a `Gate` in disguise), a side effect after success (use a `Listener`), a
retry (the input is moved into `next`; a retry must start from a new
`Actions::run`).

A `Wrap` may skip `next` only for a simulation, and then it must build the
output itself through the phase API. It cannot build an `Authorized<C>`,
because `Granted` has a private constructor, so it cannot skip the access
check. A simulation is not visible to `Listener` and `Observer`: they see a
success.

Several `Wrap` on the same stage nest. The first registered is the outermost.

### Listener

A typed side effect after one stage of one command succeeded. Signature:
`listen(&self, output: &S::Out)`. Returns nothing.

Use it for: a notification, an email, a webhook call, a cache invalidation
for one entity, a search index update. It reads the output and the caller
from the same value. It runs before `Actions::run` returns.

Do not use it for: a decision (too late, the work is done), a write that must
be consistent with the stage's write (put it in the `commit` closure or in a
database trigger; a listener runs outside the transaction and a crash
between the two loses it), a record for every command (use an `Observer`).

A `Listener` handles its own errors. It logs and returns; it has no way to
fail the action and must not panic.

### Observer

An erased record of what happened, on success and on failure. Signature:
`observe(&self, stage: StageKind, action: &dyn Action, outcome: Result<(),
&DomainError>)`. Returns nothing.

Use it for: an audit journal, metrics, usage counters, an event export or an
outbox writer, anything that must see every attempt.

On success, `action` is the output phase of the stage. On a `Policy` or
`Gate` veto, `action` is the input phase. On a `Run`, `Around` or `Wrap`
error, the input was consumed, so `action` is the envelope: id, time, caller
and access are there, and `downcast_ref` to a phase returns `None`. An
observer that counts must look at `outcome` first.

Do not use it for: a decision, a per-command effect with typed data (use a
`Listener`; a `downcast_ref` in an observer is acceptable for one field, such
as the organization of a deleted project, but a chain of downcasts means the
hook wants to be a `Listener`).

### Failure and order

- A `Policy` or a `Gate` that returns `Err` stops the action. Nothing to its
  right runs except the observers, no later stage runs, and `Actions::run`
  returns the error.
- A `Run`, an `Around` or a `Wrap` that returns `Err` stops the action. The
  `Listener` of that stage does not run; the observers run with the error.
- `Listener` runs only on success. `Observer` runs on both and cannot change
  the outcome.
- Within one position, hooks run in registration order. The order between
  extensions is the order of `Hooks::with` (or `Server::extension`) calls in
  the binary; `Feature::hooks` runs after the builder's `extension` calls.

### Decision table

| I want to                                        | position                      |
|--------------------------------------------------|-------------------------------|
| refuse a class of actions (quota, plan, flag)    | `Policy` on `Prepare`         |
| refuse before the access check (rate limit)      | `Policy` on `Authorize`       |
| refuse one action on its fields                  | `Gate<Prepare<C>>`            |
| refuse one action on its staged value            | `Gate<Persist<C>>`            |
| time, trace, count failures, for every action    | `Around` on each kind         |
| rate-limit reads and writes together             | `Policy` on `Authorize`       |
| cache one read                                   | `Wrap<Fetch<Q>>`              |
| simulate a write without writing                 | `Wrap<Persist<C>>`, skips next|
| notify after one action                          | `Listener<Persist<C>>`        |
| record every attempt, with its result            | `Observer` on each kind       |
| count what was stored                            | `Observer` on `Persist`       |

## Extensions

An extension is a type that implements `Extension` with
`register(self: &Arc<Self>, hooks: &mut Hooks)`. The receiver is the `Arc`,
so the same instance registers itself at several positions with
`self.clone()`, and the binary keeps the `Arc` to read the extension's state.

```rust
impl Extension for MeteredQuota {
    fn register(self: &Arc<Self>, hooks: &mut Hooks) {
        hooks
            .policy(StageKind::Prepare, self.clone())
            .observe(StageKind::Persist, self.clone());
    }
}
```

A binary registers it with `Server::extension(&Arc<E>)`, or a
`scylla_server::Feature` does it from `Feature::hooks`. The Community Edition
registers nothing.

## Versions

A row that `Prepare` reads and `Persist` writes carries a `version`. The store
writes an update or a delete only if the stored version is the staged one and
increments it on an update. If the row changed between the read and the
write, the write fails with `DomainError::Conflict` and nothing is written.
The caller starts again with a new `Actions::run`; a `Wrap<Persist<C>>` cannot
retry, because its input is the stale staged value. A `Gate<Persist<C>>` that
decided on the staged value is therefore safe: the write goes through only if
the row is still in the state the gate saw.

## Limits and follow-ups

- The project, organization, user, secret, pipeline, trigger, app, agent, job,
  job log, invitation, grant and role use cases are on the pipeline. The
  session (`Login`, `ValidateToken`, `RevokeToken`), `Signup`, the OAuth flow
  (`GetAuthUrl`, `OAuthCallback`), `IssueAppToken`, `AcceptInvitation` and
  `IngestWebhook` are on it too, as `Public` actions. The writes of the server
  drivers and of the agent stream are on it too. Every use case is on the
  pipeline.
- A `Public` action runs as `Anonymous`, so a `Policy` on `Authorize` sees
  every sign-in, signup and webhook delivery. The check that the use case does
  itself (the password, the app secret, the OAuth code, the invitation token,
  the webhook signature) is in `Prepare`, before the write. The `Debug` rule
  of a command with a secret is in "Adding a command or a query".
- Every sign-in path stages its session with `auth::new_session`, and a
  signup and a first OAuth login build the account with
  `signup::NewAccount`. The `commit` closure writes the account, then reloads
  the policy, then stores the session, as before.
- `ValidateToken` is a query, and its `Fetch` only reads: an expired
  session gives `false` and stays in the store. A failed read also gives
  `false`. `PurgeExpiredSessions` deletes the expired sessions: it is a pass
  that `SessionSweeper` (`session-sweeper`) sends one time each hour.
- `IngestWebhook` gives `NotFound` for an unknown, disabled or non-webhook
  trigger and `Unauthorized` for a missing or wrong signature. Every other
  failure becomes `Internal`, so the route answers 404, 401 or 500 as before.
- The `AuthInterceptor` stays outside the pipeline. It is not an action: it
  makes the caller that the actions get. It only reads. It and
  `ValidateToken` use the same session rule, `auth::look_up_session`. A
  failed read is `INTERNAL` in the interceptor and `false` in
  `ValidateToken`. The interceptor also accepts an app token.
- The agent stream sends each write through `Actions`, as the agent's own
  token: `RecordJobStatus` and `AppendJobLog` for each report (the
  `WriteJobStatus` and `AppendJobLog` checks are the authorize stage), and
  `TouchAgent` and `RecordAgentHost` for the heartbeat and the host report. No
  permission gets to the agent's own App, because its grant is on an
  organization or a project. Thus these two are `Authenticated`, the target is
  the caller, and `Prepare` refuses a caller that is not an App
  (`application::actions::app_only`). The stream sends `TouchAgent` before it
  registers the connection: if the action fails, the stream is refused with
  its status and the agent is not registered. The stream opens and closes
  the live log of a job, and frees the agent slot, only after
  `RecordJobStatus` is written. The read loop, the live fan-out
  (`InMemoryJobLogStream::publish`) and the registry stay outside the
  pipeline.
- `TailJobLogs` is a query: its `Fetch` returns the stream, so hooks see the
  subscription open, not each line. `JobLogLiveStream` is `Sync` for that
  reason, because a query output is.
- `ListJobLogs` with a node is `RequiresAll` of `ReadJobLogs` and `ReadJob`.
  Its `Fetch` reads the job through the job port and applies
  `Job::logs_readable_for`: a node that is not readable yet gives an empty
  page. `TailJobLogs` with a node applies the same rule: it replays no stored
  line for that node.
- Each write of a server driver goes through `Actions`, as a sealed
  `CallerContext::Service`, with one identity for each driver: `JobReaper`
  (`job-reaper`) sends `ReapOrphanedJobs`, `PendingJobScheduler`
  (`job-dispatcher`) sends `DispatchPendingJobs`, `TriggerCronScheduler`
  (`cron-scheduler`) sends `ScheduleCronTriggers` and `ClaimDueTriggers`,
  `TriggerFirer` (`trigger-firer`) sends `ResolveTriggerRun` and
  `RecordTriggerFire`, and `SessionSweeper` (`session-sweeper`) sends
  `PurgeExpiredSessions`. The loops stay outside the pipeline: they are not
  requests.
- A pass over many rows (`ReapOrphanedJobs`, `DispatchPendingJobs`,
  `ScheduleCronTriggers`, `ClaimDueTriggers`, `PurgeExpiredSessions`) has no
  resource for Cedar. Thus it is `Authenticated`, and `Prepare` refuses a
  caller that is not a service (`application::actions::service_only`). The
  read `ResolveTriggerRun` does the same in its `Fetch`. The guards on the
  kind of caller are together in `application/actions.rs`: `service_only`,
  `app_only` and `user_or_app` (the origin of a `RunPipeline`). A pass that
  runs each 15 or 30 seconds does not write an audit row each time. A write
  on one row asks for the permission of that row, as the bootstrap does:
  `RecordTriggerFire` asks for `ManageTrigger`, and the `service` rule of the
  Cedar policies permits it. That check writes one audit row for each fire.
- A run gives its job to an agent in its `commit` closure, after the job row
  is written: `DispatchUseCases::place` sends the job to an agent and records
  that agent. `RunPipeline`, `RunPipelineWithInputs` and `DispatchPendingJobs`
  use it. A failed attribution is logged and does not fail the command: the job
  exists and an agent has it. `place` asks for `ExecuteJob` for each agent only
  to choose one, never to refuse.
- `DeleteApp` and `SetAppActive` stage the id alone: the row is not read
  before the write, so a missing app behaves as before.
- `AcceptInvitation` is `Public`. The invitee has no account yet: the token
  is the credential, and no permission is asked. The invite mail is sent in
  the `commit` closure of `CreateInvitation`, after the write; a failed send
  is logged and does not fail the call, as before.
- `ListGrantableRoles` and `GetMyPermissions` are `Authenticated` queries: the
  catalog is static data, and a caller reads its own grants. The gRPC
  interceptor refuses a call without a token, so `Anonymous` does not get to
  them from an RPC. `GetMyPermissions` refuses a service caller in its `Fetch`
  runner: a service holds no grants, and an empty list would read as "no
  permissions".
- `CreateGrant` builds the grant in `Prepare` and checks there, in this order,
  the role, the organization admission and the escalation rule. The bootstrap
  admin grant goes through `Actions` as the bootstrap service.
- `RoleUseCases` lives in `scylla-core` (`application/role/`), not in
  `scylla-auth`: the `Run` impls need the struct in the crate that names the
  commands. `scylla-auth` keeps `Role`, `RoleRepository` and
  `validate_role_permissions`.
- `DeleteSecret` asks for its permission on the secret, not on the project.
  The access model finds the project of the secret in `Authorize`
  (`ResourceRef::Secret`, one join in `PgAuthzEntityProvider`). An unknown
  secret is under System only: without a System grant the caller gets
  `Forbidden`; with a System grant, `Prepare` gives `NotFound`. An action whose
  permission is on a parent that only the loaded row knows can use the same
  method.
- `GetTrigger`, `UpdateTrigger`, `SetTriggerEnabled`, `DeleteTrigger` and
  `FireTriggerNow` use the same method: they ask for their permission on the
  trigger (`ResourceRef::Trigger`, one join to the pipeline). `ManageTrigger`
  and `RunTriggerPipeline` have the keys `manageTriggers` and `runPipeline`, so
  the same roles give them. They are not in the permission catalog, because a
  key is there one time only. `CreateTrigger` and `ListPipelineTriggers` keep
  `ManageTriggers` on the pipeline.
- `RevokeInvitation` uses the same method: it asks for `RevokeInvitation` on
  the invitation (`ResourceRef::Invitation`, one read to the organization).
  It has the key `manageInvitations`, so the same roles give it, and it is not
  in the permission catalog. `CreateInvitation` and `ListInvitations` keep
  `ManageInvitations` on the organization.
- `RevokeAppSecret` and `SetAppSecretEnabled` use the same method: they ask for
  `ManageAppSecret` on the secret (`ResourceRef::AppSecret`, one join to the
  app and its organization). It has the key `deleteApp`, so the same roles give
  it, and it is not in the permission catalog. `CreateAppSecret` and
  `ListAppSecrets` keep their permission on the app.
- `RevokeGrant` uses the same method: it asks for `RevokeGrant` on the grant
  (`ResourceRef::Grant`, one read of the grant scope, with a join to the
  organization of a project scope). The access model then applies the
  manage-grants action of the scope kind: `manageProjectGrants`,
  `manageOrgGrants` or `manageSystemGrants`. The key of `RevokeGrant` is
  `manageSystemGrants`, the key of an unknown grant, and it is not in the
  permission catalog. An unknown grant is under System: without a
  `manageSystemGrants` grant the caller gets `Forbidden`; with one, the call
  succeeds and changes nothing, as before. `CreateGrant`, `RevokeAllAccess`
  and `ListGrants` keep their permission on the scope.
- `CreateTrigger` is `RequiresAll` of `ManageTriggers` and `RunPipeline` on
  the pipeline, and `UpdateTrigger` is `RequiresAll` of `ManageTrigger` and
  `RunTriggerPipeline` on the trigger: managing triggers must not give run
  rights. Both checks are in `Authorize`, so a `Policy` on `Prepare` runs
  after them. The runner app of the organization is provisioned in `Persist`,
  next to the trigger write.
- `FireTriggerNow` fires in its `commit` closure through the `TriggerFiring`
  port, as a scheduled fire does. `IngestWebhook` records the delivery and
  fires in its `commit` closure. `TriggerFirer` is that port for the cron
  scheduler, the webhook ingress and `FireTriggerNow`. It reads the trigger
  and the trigger-runner App of the organization with `ResolveTriggerRun`,
  which refuses a disabled trigger. Then it sends `RunPipelineWithInputs` as
  that App (one `RunPipeline` check), then sends `RecordTriggerFire`,
  best-effort. A runner App that is not found is a failed run: the fire
  records it. `FireTriggerNow` stages only the id, because the fire reads the
  trigger. `TriggerFirer` holds `Actions`, thus it is a driver and not a use
  case: a use case gets to it through the port.
- `RunPipelineWithInputs` is the fire path of a run: the origin is the trigger
  and the inputs come from the trigger, so no RPC sends it. `RunPipeline` is
  the RPC path.
- `RecordTriggerFire` carries the trigger row that `ResolveTriggerRun` read,
  and writes it back with its observation, as before.
- A `scylla_server::Feature` gets `actions` in its `Context`. An installed
  service authorizes through `Actions::run`, as the core does, so its
  actions go through the hooks. `permissions` stays in the `Context` until
  the Enterprise features use `actions`; `policy_control` and `visibility`
  serve a scope in a `Fetch`.
- The policy reload after a create or a delete sits inside the `commit`
  closure, so a failed reload still fails the call, as before. It is a
  `Listener<Persist<C>>` once a failed reload may only be logged.
- `DomainError::Conflict` maps to gRPC `ALREADY_EXISTS`; a stale write wants
  `ABORTED`, which needs a dedicated variant.
- The Cedar entity provider loads the resource's ancestors during `Authorize`;
  a missing resource surfaces there, before the permission decision.
