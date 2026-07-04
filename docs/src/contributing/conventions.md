# Conventions

The patterns to follow when adding backend code. They keep the domain pure,
testable, and hard to misuse. The shape they serve is
[the hexagonal backend](../architecture/backend.md).

## Domain modeling

- **Value objects enforce their invariant in the constructor.** A validated
  wrapper (`NodeId`, `PipelineName`, `EnvKey`, `CronSpec`) is built through a
  fallible constructor and is immutable — if you hold one, it's valid, so
  downstream code never re-checks. Put validation *there*, not in handlers.
- **Entities own their rules.** Business invariants (DAG validity, state
  transitions) live on the entity (`Pipeline::create`, `Job` status machine), not
  in the use case or handler.
- **The domain imports no I/O.** No sqlx, no Cedar, no tonic in `domain/`. If the
  domain needs something from outside, it's a **port**.

## Use cases & ports

- One **use-case** struct per aggregate (`PipelineUseCases`, `SecretUseCases`, …),
  holding `Arc<dyn Port>` dependencies and exposing async methods.
- Every state-changing (and most read) methods **authorize first**:
  `permission_service.check(caller, Permission::…).await?` before doing work.
- Depend on the **trait**, never a concrete adapter — that's what makes use cases
  unit-testable with stubs (see `test_support/`).

## Errors

- Return `DomainResult<T>` from core, using `DomainError` variants
  (`validation`, `business_rule`, `not_found`, …).
- gRPC **handlers** in `scylla-api` map `DomainError` to the right gRPC `Status`;
  don't leak transport types into core.
- Authorization is `DomainResult<()>`, never `bool` — fail-closed by construction.

## Adding a repository

1. Define the port trait in the aggregate's `application/<area>/repository.rs`.
2. Implement `Pg<Aggregate>Repository` under
   `infrastructure/persistence/postgres/<aggregate>/`, using `sqlx::query!` /
   `query_as!` (compile-time checked).
3. Run `just db-prepare` and commit the updated `.sqlx/`.
4. Wire the adapter into the `Services` composition in `scylla-api`.

## Naming & vocabulary

- **Authorization uses the seven words exactly** (Permission, Role, Scope, Grant,
  Principal, Caller, Policy) — in code, proto, DB, and UI. Don't reintroduce
  "action" or coin synonyms. See [the authorization model](../architecture/authorization.md).
- **Builtin role names are `<scope>-<role>`**, kebab-case (`organization-admin`).
  The `*_ROLE` constants in `application/authz/grant.rs` are the single source of
  truth.
- Frontend types follow their own `*.entity.ts` / `*.struct.ts` rule — see
  `apps/frontend/docs/naming-conventions.md`.

## Lints

The workspace runs `clippy::all` + `pedantic` with `unsafe_code = warn`. A few
pedantic lints are deliberately relaxed in `Cargo.toml`; don't re-fight those, but
keep new code warning-clean.
