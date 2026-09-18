# scylla-extension: the action pipeline

Every write in Scylla is a command that moves through three typed stages, and
an extension attaches to each stage. This crate holds the pipeline. It depends
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
    command.rs     Command: Staged, Committed, permission()
    value.rs       Draft<T> (not stored), Deleted<T> (tombstone)
    envelope.rs    Envelope<C>: id, at, caller, permission, command; built once, shared by Arc
    phase.rs       Requested, Authorized, Prepared, Committed; Prepared::commit
    erased.rs      Action: the erased view of any phase; Phase: gives the envelope
  stage.rs       StageKind, Stage, Authorize/Prepare/Persist<C>, Run<S>
  authz.rs       Granted (private constructor), Authorizer, AuthorizeStage
  hooks/         the extension seam
    position.rs    Policy, Gate, Around, Wrap, Listener, Observer
    next.rs        Next: the rest of a typed chain; Proceed and Done: the erased one
    registry.rs    Hooks: registration and run()
    extension.rs   Extension: register(self: &Arc<Self>, &mut Hooks)
  actions.rs     Actions: send(runner, caller, command) through the three stages
  tests.rs       the checks, on a fake aggregate
```

The core side lives in `scylla-core`: `application/actions.rs` adapts
`PermissionService` to `Authorizer`, and `application/project/{commands,
prepare, persist}.rs` is the reference use case.

## The event

One action is one event. It is born in `Requested<C>`, where `C` is the
command. `Requested<C>` holds an `Arc<Envelope<C>>`: the action id, the time,
the caller, the permission and the complete command. Every later phase keeps
the same envelope. All phases deref to it, so `caller()`, `permission()`,
`command()` and `id()` are available at each step. No phase has a public
constructor. The only way to get a later phase is a transition method on the
earlier phase, and each transition consumes its input.

| phase           | new data                       | who produces it       |
|-----------------|--------------------------------|-----------------------|
| `Requested<C>`  | envelope                       | `Actions::send`       |
| `Authorized<C>` | nothing; `Granted` is consumed | the authorize stage   |
| `Prepared<C>`   | `C::Staged`                    | the use case          |
| `Committed<C>`  | `C::Committed`                 | the store             |

`Granted` is a token with a private constructor and no data. Only
`AuthorizeStage` makes one, and `Requested::authorized` consumes it, so an
`Authorized<C>` is proof that the permission check ran.

The command declares its two payload types and its permission:

```rust
pub struct CreateProject { pub organization_id: OrganizationId, pub name: ProjectName, ... }

impl Command for CreateProject {
    type Staged = Draft<NewProject>;
    type Committed = Project;

    fn permission(&self) -> Permission {
        Permission::CreateProject(self.organization_id.clone())
    }
}
```

A `Draft<T>` is a value that is not in the store. A `Project` that comes out of
`Committed<CreateProject>` is in the store. For `DeleteProject`, `Staged` is
the loaded `Project` and `Committed` is `Deleted<Project>`, a tombstone with
the last state. The type tells you if the thing exists.

`Prepared<C>` has one way out: `commit(async |staged| ...)`. The closure gets
the staged value by move and must return a `C::Committed`. The store writes
inside the closure. A closure that does not write is a simulation, which is
what a dry-run `Wrap` does.

## The stages

Three stages move the event. Each is a type with an `In` and an `Out` phase:

- `Authorize<C>`: `Requested<C>` to `Authorized<C>`. `AuthorizeStage` runs it
  for every command; it asks the `Authorizer` (in the core, the Cedar
  `PermissionService`) and mints the `Granted`.
- `Prepare<C>`: `Authorized<C>` to `Prepared<C>`. The use case runs it. It
  builds the draft or loads the target. It reads, it does not write.
- `Persist<C>`: `Prepared<C>` to `Committed<C>`. The use case runs it too,
  against its repository port, by calling `commit` and writing inside the
  closure.

`Run<S>` is the trait a stage implementation satisfies:
`async fn run(&self, input: S::In) -> DomainResult<S::Out>`. The signature is
fixed by `S`; the compiler, not the author, decides the phases.

`Actions::send(runner, caller, command)` chains the three stages through
`Hooks::run`. The runner is one object that implements `Run<Prepare<C>>` and
`Run<Persist<C>>`; a use case passes `self`. There is one `Actions` per
server, built in `init_services` with the permission service and the hooks.

The permission check is a stage and not a hook for two reasons. It runs with
zero hooks registered, because `send` calls it in code; a hook is something a
binary may or may not register. And it has an output, `Authorized<C>`, that a
`Gate` cannot produce: the use case's `Run<Prepare<C>>` takes `Authorized<C>`,
so its signature is the proof that the check ran.

### Adding a command to a use case

Three files and no hook code:

1. `commands.rs`: the struct with public fields, `impl Command` with the
   permission and the two payload types.
2. `prepare.rs`: `impl Run<Prepare<C>> for XUseCases`. Read through the port,
   build the domain value, `input.prepared(staged)`.
3. `persist.rs`: `impl Run<Persist<C>> for XUseCases`.
   `input.commit(async |staged| self.repo.write(&staged).await).await`.

Then one public method on the use case:

```rust
pub async fn create(&self, caller: &CallerContext, command: CreateProject) -> DomainResult<Project> {
    self.actions.send(self, caller, command).await.map(Committed::into_outcome)
}
```

Reads (`get`, `list`) are not commands and take no hooks.

## The hook positions

`Hooks::run` surrounds each stage with six positions. They run in this order:

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

Use it for: a cache in front of the stage, a dry-run mode, anything that needs
the typed input or output. It calls `next.run(input)` exactly once in normal
operation. A dry run is a `Wrap<Persist<C>>` that never calls `next` and builds
its output with `input.commit(async |draft| Ok(draft.into_inner()))`.

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
| time, trace, count failures, for every command   | `Around` on each kind         |
| cache one command, simulate without writing      | `Wrap<S>`                     |
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

- The project use case is the only one on the pipeline. The other aggregates
  keep their hand-written sequence until they migrate.
- The policy reload after a create or a delete sits inside the `commit`
  closure, so a failed reload still fails the call, as before. It is a
  `Listener<Persist<C>>` once a failed reload may only be logged.
- `DomainError::Conflict` maps to gRPC `ALREADY_EXISTS`; a stale write wants
  `ABORTED`, which needs a dedicated variant.
- The Cedar entity provider loads the resource's ancestors during `Authorize`;
  a missing resource surfaces there, before the permission decision.
