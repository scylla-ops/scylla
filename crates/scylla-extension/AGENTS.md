# scylla-extension: the action pipeline

Every write in Scylla is a command that moves through three typed stages,
every read is a query that moves through two, and an extension attaches to
each stage. This crate holds the pipeline. It depends
on `scylla-domain` and nothing else: no access model, no database, no gRPC, no
Cedar. A hook sees `CallerContext`, `Permission` and `DomainError`, never a
repository or a wire type. An Enterprise build compiles it from a pinned tag.

Run `cargo test -p scylla-extension` for the in-crate checks. They drive the
engine with a self-contained aggregate (`src/tests.rs`), so a change to the
pipeline is proven here before it reaches a use case.

## Layout

```
src/
  action/        the event
    id.rs          ActionId: one per send
    command.rs     Describe: permission(); Command: Staged, Committed; Query: Output
    value.rs       Draft<T> (not stored), Deleted<T> (tombstone)
    envelope.rs    Envelope<C>: id, at, caller, permission, command; built once, shared by Arc
    phase.rs       Requested, Authorized, Prepared, Committed, Fetched; Prepared::commit
    erased.rs      Action: the erased view of any phase; Phase: gives the envelope
  stage.rs       StageKind, Stage, Authorize/Prepare/Persist<C>, Fetch<Q>, Run<S>
  authz.rs       Granted (private constructor), Authorizer, AuthorizeStage
  hooks/         the extension seam
    position.rs    Policy, Gate, Around, Wrap, Listener, Observer
    next.rs        Next: the rest of a typed chain; Proceed and Done: the erased one
    registry.rs    Hooks: registration and run()
    extension.rs   Extension: register(self: &Arc<Self>, &mut Hooks)
  path.rs        Path<K, R>: the chain after Authorize, one impl per marker (Write, Read)
  actions.rs     Actions: run(runner, caller, action) for a command or a query
  tests.rs       the checks, on a fake aggregate
```

The core side lives in `scylla-core`: `application/actions.rs` adapts
`PermissionService` to `Authorizer`, `application/project/` is the reference
use case (`commands.rs`, `queries.rs`, the ports in `mod.rs`), and
`grpc/handlers/project_handler.rs` is the reference adapter.

## The event

One action is one event. It is born in `Requested<C>`, where `C` is the
command or the query. `Requested<C>` holds an `Arc<Envelope<C>>`: the action
id, the time, the caller, the permission and the complete command. Every later phase keeps
the same envelope. All phases deref to it, so `caller()`, `permission()`,
`command()` and `id()` are available at each step. No phase has a public
constructor. The only way to get a later phase is a transition method on the
earlier phase, and each transition consumes its input.

| phase           | new data                       | who produces it       |
|-----------------|--------------------------------|-----------------------|
| `Requested<C>`  | envelope                       | `Actions`             |
| `Authorized<C>` | nothing; `Granted` is consumed | the authorize stage   |
| `Prepared<C>`   | `C::Staged`                    | the use case          |
| `Committed<C>`  | `C::Committed`                 | the store             |
| `Fetched<Q>`    | `Q::Output`                    | the use case          |

`Granted` is a token with a private constructor and no data. Only
`AuthorizeStage` makes one, and `Requested::authorized` consumes it, so an
`Authorized<C>` is proof that the permission check ran.

A command or a query declares its permission through `Describe`, and its
payload types through `Command` or `Query`. Nothing else: which path it takes
follows from the trait.

```rust
pub struct CreateProject { pub organization_id: OrganizationId, pub name: ProjectName, ... }

impl Describe for CreateProject {
    fn permission(&self) -> Permission {
        Permission::CreateProject(self.organization_id.clone())
    }
}

impl Command for CreateProject {
    type Staged = Draft<NewProject>;
    type Committed = Project;
}

pub struct GetProject { pub id: ProjectId }

impl Describe for GetProject {
    fn permission(&self) -> Permission {
        Permission::ReadProject(self.id.clone())
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
  for every command and query; it asks the `Authorizer` (in the core, the
  Cedar `PermissionService`) and mints the `Granted`.
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
and every action runs in one tracing span with its kind, id, caller,
permission and resource; use cases carry no `#[instrument]` of their own.

The permission check is a stage and not a hook for two reasons. It runs with
zero hooks registered, because `send` calls it in code; a hook is something a
binary may or may not register. And it has an output, `Authorized<C>`, that a
`Gate` cannot produce: the use case's `Run<Prepare<C>>` takes `Authorized<C>`,
so its signature is the proof that the check ran.

### The use case and the adapter

A use case struct (`ProjectUseCases`) holds the ports and implements the `Run`
traits. Each port is an `Arc<dyn Port>`, thus the struct, its `Run` impls and
its handler have no type parameters. It has no method of its own, no `Actions`
field, no permission code and no hook code. The adapter (a gRPC handler) holds
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
permission, `impl Command` with the two payload types, `impl Run<Prepare<C>>`
(read through the port, build the domain value, `input.prepared(staged)`),
`impl Run<Persist<C>>` (`input.commit(async |staged| self.repo.write(&staged).await).await`).
Inside `queries.rs`, one block per query: the struct, `impl Describe`,
`impl Query` with the output type, `impl Run<Fetch<Q>>` (read,
`input.fetched(output)`). A new action is one block in the right file, and
the outline of the file is the list of actions.

A `tests.rs` gets its engine from `test_support::authz::actions(permissions)`:
the `PermissionAuthorizer` and no hooks. Use `actions_with(permissions, hooks)`
when the test registers hooks. The stubs that more than one use case needs are
in `test_support::stubs` (compiled for tests only): `CountingPolicy`,
`StubHash`, `StubRegistry`, `StubRoles`, `StubGrants`, `NoUsers`, `OneProject`,
`OnePipeline`, `EchoResolver`, `empty_page` and `alice`. Keep a stub in the
`tests.rs` of the use case when its behavior is specific to that use case.

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
`at()`, the caller, the `Permission`, and `downcast_ref` to the typed phase.
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
flags, kill switches. The decision reads `action.permission()`,
`action.caller()`, and state the extension owns. A quota returns
`DomainError::quota_exceeded(..)`, which maps to `RESOURCE_EXHAUSTED`.

Do not use it for: a rule that needs one command's fields (use a `Gate`), a
side effect (use an `Observer`), anything that writes.

Which `StageKind`:

- `Authorize`: runs before the permission check. Only for rules that must
  apply to unauthorized callers too, for example a rate limit or a
  maintenance mode. A quota here leaks quota state to callers who have no
  right to create.
- `Prepare`: runs after the permission check and before the use case. The
  default position for a policy.
- `Persist`: runs after the use case and before the store. Use it when the
  rule needs the staged value, reached through `downcast_ref`.
- `Fetch`: runs after the permission check and before the read. A rate limit
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
Signature: `wrap(&self, input: S::In, next: Next<'_, S>) -> DomainResult<S::Out>`.

Use it for: a cache in front of a read, a dry-run mode, anything that needs
the typed input or output. It calls `next.run(input)` exactly once in normal
operation. A read cache is a `Wrap<Fetch<GetProject>>` that answers with
`input.fetched(cached)` on a hit and stores `fetched.output()` on a miss. A dry
run is a `Wrap<Persist<C>>` that never calls `next` and builds its output with
`input.commit(async |draft| Ok(draft.into_inner()))`.

Do not use it for: a business rule (a `Wrap` that returns `Err` to refuse is
a `Gate` in disguise), a side effect after success (use a `Listener`), a
retry (the input is moved into `next`; a retry must start from `send`).

A `Wrap` may skip `next` only for a simulation, and then it must build the
output itself through the phase API. It cannot build an `Authorized<C>`,
because `Granted` has a private constructor, so it cannot skip the permission
check. A simulation is not visible to `Listener` and `Observer`: they see a
success.

Several `Wrap` on the same stage nest. The first registered is the outermost.

### Listener

A typed side effect after one stage of one command succeeded. Signature:
`listen(&self, output: &S::Out)`. Returns nothing.

Use it for: a notification, an email, a webhook call, a cache invalidation
for one entity, a search index update. It reads the output and the caller
from the same value. It runs before `send` returns.

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
and permission are there, and `downcast_ref` to a phase returns `None`. An
observer that counts must look at `outcome` first.

Do not use it for: a decision, a per-command effect with typed data (use a
`Listener`; a `downcast_ref` in an observer is acceptable for one field, such
as the organization of a deleted project, but a chain of downcasts means the
hook wants to be a `Listener`).

### Failure and order

- A `Policy` or a `Gate` that returns `Err` stops the action. Nothing to its
  right runs except the observers, no later stage runs, and `send` returns
  the error.
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
| refuse before the permission check (rate limit)  | `Policy` on `Authorize`       |
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
The caller starts again from a fresh `send`; a `Wrap<Persist<C>>` cannot
retry, because its input is the stale staged value. A `Gate<Persist<C>>` that
decided on the staged value is therefore safe: the write goes through only if
the row is still in the state the gate saw.

## Limits and follow-ups

- The project, organization, user, secret, pipeline, trigger, app, agent, job,
  job log, invitation, grant and role use cases are on the pipeline. The other aggregates
  keep their hand-written sequence until they migrate.
- The agent stream sends `RecordJobStatus` and `AppendJobLog` through
  `Actions` for each report, as the agent's own token, so the `WriteJobStatus`
  and `AppendJobLog` checks are the authorize stage. The live fan-out
  (`InMemoryJobLogStream::publish`) and the rest of the stream stay outside the
  pipeline.
- `TailJobLogs` is a query: its `Fetch` returns the stream, so hooks see the
  subscription open, not each line. `JobLogLiveStream` is `Sync` for that
  reason, because a query output is.
- `ListJobLogs` with a node sends `GetJob` first, from the handler. That
  `ReadJob` check refuses, so it is not a scope inside `Fetch`; a node that is
  not readable yet gives an empty page, as before.
  `JobReaper` writes through the port directly: it runs as the server, not for
  a caller.
- `DispatchUseCases`, `PendingJobScheduler` and the agent stream
  (`AgentHandler`) stay outside the pipeline. They run as the scheduler or as
  the agent's own token, not for a caller that asks for a permission;
  `dispatch_job` asks for `ExecuteJob` only to choose an agent, never to refuse.
- `AppUseCases::revoke_secret` and `set_secret_enabled` stay outside the
  pipeline. Their permission is `DeleteApp` on the secret's app, and only the
  loaded credential knows that app. `DeleteApp` and `SetAppActive` stage the id
  alone: the row is not read before the write, so a missing app behaves as
  before.
- `InvitationUseCases::revoke` stays outside the pipeline. Its permission is
  `ManageInvitations` on the invitation's organization, and only the loaded
  invitation knows that organization.
- `InvitationAcceptUseCases::accept` stays outside the pipeline. The invitee
  has no account yet: the token is the credential, and no permission is asked.
  The invite mail is sent in the `commit` closure of `CreateInvitation`, after
  the write; a failed send is logged and does not fail the call, as before.
- `GrantUseCases::revoke` stays outside the pipeline. Its permission is the
  manage-grants permission of the grant's scope, and only the loaded grant knows
  that scope; an unknown id asks for `ManageSystemGrants`. `ListGrantableRoles`
  asks for no permission: the catalog is static data.
- `CreateGrant` builds the grant in `Prepare` and checks there, in this order,
  the role, the organization admission and the escalation rule. The bootstrap
  admin grant goes through `Actions` as the bootstrap service.
- `RoleUseCases` lives in `scylla-core` (`application/role/`), not in
  `scylla-auth`: the `Run` impls need the struct in the crate that names the
  commands. `scylla-auth` keeps `Role`, `RoleRepository` and
  `validate_role_permissions`. `RoleUseCases::my_permissions` stays outside the
  pipeline: a caller reads its own grants and no permission is asked.
- `SecretUseCases::delete` stays outside the pipeline. Its permission is
  `DeleteSecret` on the secret's project, and only the loaded secret knows that
  project; `Describe` sees the command alone.
- `TriggerUseCases::get`, `update`, `set_enabled` and `delete`, and
  `TriggerFireUseCases::fire_now`, stay outside the pipeline. Their permission
  is on the trigger's pipeline, and only the loaded trigger knows that pipeline.
- `CreateTrigger` asks for `RunPipeline` a second time in its `Prepare` runner.
  This check refuses: managing triggers must not give run rights. A `Policy` on
  `Prepare` therefore runs before it. The runner app of the organization is
  provisioned in `Persist`, next to the trigger write.
- `PipelineUseCases::run_with_inputs` and `assign_agent` stay outside the
  pipeline. A trigger fire calls them as the trigger-runner App, with an origin
  and inputs that no RPC sends. `RunPipeline` is the RPC path; the handler then
  gives the job to an agent, as before.
- The policy reload after a create or a delete sits inside the `commit`
  closure, so a failed reload still fails the call, as before. It is a
  `Listener<Persist<C>>` once a failed reload may only be logged.
- `DomainError::Conflict` maps to gRPC `ALREADY_EXISTS`; a stale write wants
  `ABORTED`, which needs a dedicated variant.
- The Cedar entity provider loads the resource's ancestors during `Authorize`;
  a missing resource surfaces there, before the permission decision.
