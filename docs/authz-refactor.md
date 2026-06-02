# Authorization Refactor — Unified RBAC

Status: **in progress** (started 2026-06-02, branch `poc`). This document is the
reference for the multi-phase refactor of Scylla's authorization system. It
records the target model, the locked decisions behind it, and the phased plan.

The canonical word-by-word vocabulary lives in [`GLOSSARY.md`](../GLOSSARY.md)
(section *Authorization*). This document explains the **model and the plan**; the
glossary is the **dictionary**. Keep them in sync.

---

## 1. Why

Today the system already did a lot right: one `grants` table
`(principal, role, scope)`, scopes `System / Organization / Project`, principal =
`User | App`, the global admin is just a System-scoped grant, and Cedar is the
evaluation engine. But two limits block the product vision:

1. **Roles are not composable.** A role maps to one of two *fixed* Cedar
   templates — `role_template` (admin = everything in scope) or `agent_template`
   (a hard-coded restricted set). You cannot express "reads + runs pipelines but
   cannot delete". Roles must become **editable bundles of permissions**.
2. **The vocabulary is overloaded.** `permission / grant / role / policy /
   action / authz / scope / resource / principal / caller / subject /
   assignment` blur together. We pick **one word per concept** and use it
   everywhere (code, proto, DB, UI).

## 2. Locked decisions

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | **Permission** is the canonical word for the atomic capability. | User-chosen. One word, everywhere. |
| 2 | **Permissions are a closed, code-owned, strongly-typed catalog** (a Rust enum, exposed as a gRPC `enum`). **Roles are the dynamic/creatable thing.** | A permission only means something if the backend enforces it — it can't be invented at runtime. Roles compose permissions and *can* be created. This reconciles "strong gRPC enums" with "dynamic creation". |
| 3 | **Keep Cedar as the engine; generate the Cedar policy set from the RBAC model.** | Preserves hierarchy resolution (`in`), audit, diagnostics, and the advanced-policy escape hatch already built. |
| 4 | **Default roles resolved via a configurable pointer** (`slot → role_id`), defaulting to a builtin, lazily re-seeded if missing. | Code never hard-codes a role name. Deleting/renaming `organization-admin` can't break org creation. |

Supporting goals: direct permission grants additive to roles (Alice `RunPipeline`
in Org A); introspection (`CheckPermissions`, `ListPermissions` catalog,
effective permissions of a principal); anti-lateral-movement (only delegate what
you hold + the meta-permission to delegate is reserved to system scope).

## 3. Ubiquitous language (the 7 words)

| Word | The one meaning | Rust | Replaces |
|------|-----------------|------|----------|
| **Permission** | atomic capability — a verb on a resource type (`RunPipeline`) | `Permission` enum | `Action` |
| **Role** | named, editable bundle of permissions, bound to a scope kind | `Role` (DB row) | the 2 fixed templates, `GrantableRole` |
| **Scope** | the level a grant/role binds to: System / Organization / Project | `Scope` (+ `ScopeKind` discriminant) | `GrantScope` |
| **Grant** | "principal P holds {role \| permission} within scope S" | `Grant` | `assignment` |
| **Principal** | a grant-holding actor: `User \| App` | `Principal` | `GrantPrincipal` |
| **Caller** | the request identity: a Principal, an internal Service, or Anonymous | `CallerContext` | `subject` |
| **Policy** | **only** advanced Cedar escape-hatch rules (`cedar_policies`) | `PolicyDefinition` | — |
| _(Resource)_ | the concrete entity an action targets | `ResourceRef` | — |

Notes:
- **Permission vs Caller/Principal.** `Caller` and `Principal` are *not* synonyms.
  A `Caller` is who made the request (may be a `Service` or `Anonymous`, which
  cannot hold grants). A `Principal` is specifically a grant-holding actor
  (`User | App`). Keep both; the glossary spells out the distinction.
- **"action" is retired from our vocabulary** but remains Cedar's wire term
  (`Scylla::Action::"runPipeline"`). It stays an infra detail inside the Cedar
  adapter and the `.cedar` files — it never appears in the domain/application API
  again.
- **Abstract vs concrete permission.** The runtime `Permission` enum carries the
  target id where relevant (`Permission::RunPipeline(PipelineId)`) so a `check`
  is "can P run *this* pipeline". The **abstract** permission (what a role stores,
  what the proto enum names) is its key string — `Permission::key()` →
  `"runPipeline"`. We do **not** split the runtime enum; the key is the bridge.

## 4. Target model

### 4.1 Permission catalog (code-owned)

`Permission` (renamed from `Action`) stays the single enum the application layer
uses for `check`. Each variant exposes:
- `key() -> &'static str` — the canonical id (was `action()`), e.g. `"runPipeline"`.
  Becomes the Cedar `Action::"…"` eid and the value stored in `role_permissions`.
- `resource() -> ResourceRef` — the concrete target.

`PERMISSION_CATALOG` (was `ACTION_CATALOG`) + `RESOURCE_TYPES` drive
introspection. The proto `enum Permission` has exactly one value per key.

### 4.2 Roles as data

```sql
CREATE TABLE roles (
    id            TEXT PRIMARY KEY,
    key           TEXT,           -- stable id for builtins ("organization-admin"); NULL for custom
    name          TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    scope_kind    TEXT NOT NULL,  -- system|organization|project
    owner_org_id  TEXT REFERENCES organizations(id) ON DELETE CASCADE, -- NULL = global/builtin
    builtin       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE role_permissions (
    role_id     TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission  TEXT NOT NULL,    -- a Permission::key()
    PRIMARY KEY (role_id, permission)
);
```

**Ownership (SaaS):** builtin roles are global (`owner_org_id` NULL); custom
roles are owned by an Org (tenant isolation — invisible to other tenants). A
custom role may *target* the project scope kind but is still owned by the org.

### 4.3 Generalized grant

```sql
CREATE TABLE grants (
    id              TEXT PRIMARY KEY,
    principal_kind  TEXT NOT NULL,            -- user|app
    principal_id    TEXT NOT NULL,
    target_kind     TEXT NOT NULL,            -- role|permission
    role_id         TEXT REFERENCES roles(id) ON DELETE CASCADE,
    permission      TEXT,                     -- a Permission::key()
    scope_kind      TEXT NOT NULL,
    scope_id        TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK ((target_kind='role'       AND role_id IS NOT NULL AND permission IS NULL)
        OR (target_kind='permission' AND permission IS NOT NULL AND role_id IS NULL))
);
```

A direct permission grant (Alice `RunPipeline` in Org A) is additive to her
role-derived permissions.

### 4.4 Default-role pointers

```sql
CREATE TABLE default_role_bindings (
    slot     TEXT PRIMARY KEY,   -- 'org_creation' | 'project_creation'
    role_id  TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT
);
```

Org/project creation resolve the assigned role via its slot, so the code never
names a role directly — an admin can rebind a slot to a custom role.
`ON DELETE RESTRICT` keeps a bound role from being deleted out from under its
slot (rebind first, then delete), so the pointer never dangles — simpler and
safer than a lazy re-seed. The resolver still falls back to the slot's builtin
key if a binding is somehow absent. (Bootstrap keeps assigning `system-admin`
directly for now; it could be slotted later.)

### 4.5 Cedar generation (from RBAC)

Drop `role_template.cedar` / `agent_template.cedar` (the 2 fixed shapes). Instead
generate, on each reload:
- **per role grant** → `permit(principal == P, action in [Action::"k1", …], resource in <scope>);`
  where the list is the role's `role_permissions`.
- **per direct permission grant** → `permit(principal == Alice, action == Action::"runPipeline", resource in Org_A);`
- base ABAC membership policies (`policies.cedar`) stay.
- the `cedar_policies` table stays as the advanced permit/forbid escape hatch.

Cedar still resolves the scope hierarchy via `in` and produces the audit trail.

### 4.6 Introspection (proto)

The **set of permissions and its static metadata** (resource type, valid scopes,
label) is **not** a runtime endpoint — it rides with the `enum Permission` in the
proto, which is self-describing on both sides (in Phase 4, enrich it with
protobuf *enum value options* so resource-type/scope/label are carried by the
descriptor, not duplicated in the frontend). A "list available permissions" RPC
would only re-serialize what the generated code already holds — dropped. The
existing `PolicyService.ListAuthzVocabulary` becomes redundant once permissions
are an enum and is removed in Phase 4.

Only **runtime, per-principal** introspection (which depends on DB state — grants,
roles, policies) is worth an RPC:
- `CheckPermissions(caller, [{permission, scope_kind, scope_id}]) → [{allowed}]` —
  "can *I* do these here", for conditional UI.
- `GetEffectivePermissions(principal, scope) → set` — "what can Alice do in Org A",
  the resolved union of her roles + direct grants. Powers the matrix.
- `ListGrantableRoles(scope?)` — now backed by `roles` (builtin + tenant custom).

### 4.7 Strong-typed proto

```protobuf
enum Permission    { PERMISSION_UNSPECIFIED = 0; CREATE_USER = 1; RUN_PIPELINE = 2; MANAGE_GRANTS = 3; /* 1 per key */ }
enum Scope         { SCOPE_UNSPECIFIED = 0; SYSTEM = 1; ORGANIZATION = 2; PROJECT = 3; }
enum ResourceType  { RESOURCE_TYPE_UNSPECIFIED = 0; SYSTEM = 1; USER = 2; ORGANIZATION = 3; PROJECT = 4; PIPELINE = 5; JOB = 6; APP = 7; }
enum PrincipalKind { PRINCIPAL_KIND_UNSPECIFIED = 0; USER = 1; APP = 2; }

message Grant {
  string id = 1; PrincipalKind principal_kind = 2; string principal_id = 3;
  oneof target { string role_id = 4; Permission permission = 5; }
  Scope scope_kind = 6; string scope_id = 7;
}
```

Proto enums are append-only (`UNSPECIFIED = 0`, never renumber). Each new backend
capability = one enum value + the matching `Permission` variant. That coupling is
intentional: a permission must map to enforced code.

### 4.8 Anti-lateral-movement

Enforced in the grant use case:
1. **No escalation** — a principal may grant a target at a scope only if it
   itself holds every permission in that target at that scope (subset check).
2. **Meta-permission** — conferring `MANAGE_GRANTS` (the right to delegate) is
   reserved to the System scope: only a system admin can mint new delegators.

## 5. Phased plan

Each phase compiles, keeps tests green, and is independently commitable.

- **Phase 0 — Vocabulary rename (no behavior change).** `Action → Permission`
  (+ `action() → key()`, `ACTION_CATALOG → PERMISSION_CATALOG`, module
  `value_objects::action → ::permission`), `GrantScope → Scope`,
  `GrantPrincipal → Principal`. Cedar untouched (still 2 templates). Update
  `GLOSSARY.md`. ← **current**
- **Phase 1 — Roles as data. ✅ done.** Added `roles` + `role_permissions` tables
  and seeded the 5 builtins (admins → `*`, agents → the 4 job keys);
  `RoleRepository` + `PgRoleRepository`; Cedar generation now builds one template
  per role from its permission set (full control → unconstrained action;
  otherwise an explicit key list), replacing the two compiled-in templates.
  Behaviour-equivalent (verified by the existing Cedar admin/agent tests).
  *Deferred:* the `grants` table still references a role by its key (== builtin
  id); the `role_name → role_id` column rename waits until custom roles need
  opaque ids (Phase 2/4). Grant validation still uses the compile-time
  `GRANTABLE_ROLES` catalog; it moves to the DB when role CRUD lands (Phase 4).
- **Phase 2 — Direct permission grants. ✅ done.** Generalized the `grants` table
  to `target_kind` + `target` (a grant confers a role *or* a single permission).
  `GrantTarget` enum; `Grant::with_permission`; repo + Cedar updated (role →
  template instance, permission → a direct `permit(... action == Action::"key"
  ... resource in scope)`); validation rejects unknown permission keys. The
  canonical "Alice runPipeline in Org A" is proven by a Cedar test (allows that
  action in scope, denies everything else). *Deferred to Phase 4:* exposing
  permission grants over gRPC (needs the typed `Permission` enum) and
  `GetEffectivePermissions`. The grant use case still gates on the existing
  manage-grants permission; the no-escalation rule is Phase 5.
- **Phase 3 — Default-role pointers. ✅ done.** Added `default_role_bindings`
  (`slot → role_id`, `ON DELETE RESTRICT`), seeded `org_creation → organization-
  admin` and `project_creation → project-admin`. `DefaultRoleSlot` +
  `DefaultRoleBindingRepository` + `resolve_default_role` (falls back to the
  builtin key). Rewired org/project creation, signup and OAuth onboarding to
  resolve via the slot instead of a hard-coded role name; an admin can rebind a
  slot to a custom role (proven by tests). Bootstrap still assigns `system-admin`
  directly.
- **Phase 4a — Strong-typed proto common value objects. ✅ done.** `common.proto`
  wrapper messages for every id (UserId, OrganizationId, …, NodeId, JobLogId) +
  Email, with solid docs (ids are lowercase ULIDs); timestamps →
  `google.protobuf.Timestamp` (added `prost-types`). A `grpc::convert` helper
  module (`wrap`/`required`/`optional`/`ts`/`dt`) keeps mappers/handlers terse.
  Every service migrated except `permission.proto` (folded into 4b) and
  `config`/`agent` (no ids). Frontend regen works via `pnpm i && pnpm run
  gen-proto`; frontend **app** code still needs unwrapping `.value`/Timestamp.
- **Phase 4b.1 — Typed authz enums. ✅ done.** `permission.proto` now has the
  closed `Permission` enum (mirrors the catalog), `Scope`, `ResourceType`,
  `PrincipalKind`; `ListAuthzVocabulary` returns typed Permission + ResourceType
  (the matrix source); grants use `Scope` + `common.UserId`; Policy timestamps →
  Timestamp. `grpc::convert` gains `permission_key`/`permission_from_key`/
  `resource_type_from_tag` (camelCase ⇄ SCREAMING_SNAKE) with a sync test that
  keeps the enum a total mirror of the catalog.
- **Phase 4b.2 — RoleService (dynamic role CRUD). ✅ done.** New `manageRoles`
  permission; `RoleRepository` writes + `RoleUseCases` (validate permission set,
  reload on change, refuse deleting builtins / still-granted roles); proto
  `RoleService` (Create/Update/Delete/List/GetRole) with a typed
  `Role { Scope, repeated Permission, full_control }`; handler + wiring. First
  cut: global roles, system-admin only.
- **Phase 4b.3 — introspection (`GetEffectivePermissions`). ✅ done.**
  `RoleService.GetEffectivePermissions(principal)` resolves a principal's grants
  into permissions grouped by scope (roles expanded + direct grants; `*` →
  full_control) — the matrix source. Gated by `manageGrants`.
- **Phase 5 — Anti-escalation. ✅ done.** A grant's creator may only confer
  permissions it already holds at the scope (subset check; full control may only
  be conferred by a full-control holder). Closes the lateral-movement hole.
  System + Organization scopes; Project deferred.
- **Direct permission grants over gRPC. ✅ done.** `CreateGrant`/`Grant` gained an
  optional `permission` field (additive, frontend-safe). "Alice runPipeline in
  Org A" is now grantable via the API; same anti-escalation applies.
- **Frontend reconciliation. ✅ done.** The proto big-bang broke ~96 app-code
  typecheck errors at the gRPC boundary; reconciled with a small wrapper helper
  so existing features keep working. `pnpm i && pnpm run gen-proto && pnpm run
  typecheck` → 0, build OK.
- **Remaining thin slices (post-MCP-testing).** Org-owned custom roles + org-admin
  management (`ManageOrgRoles`, `owner_org`); `ListGrantableRoles` from DB (custom
  roles in the assignable list — needs a `GrantableRole` id/name split);
  `CheckPermissions` per-resource probe (needs key→domain-Permission construction);
  App principals as grant targets; Project-scope anti-escalation.
- **Phase 5 — Anti-escalation + meta-permission rules.**
- **Phase 6 — Frontend permission matrix** (documented only — the frontend does
  not build in this environment; see memory `project_frontend_codegen_blocker`).

## 6. Open sub-decisions (default chosen, revisit if needed)

- Custom roles owned by an **Org** (+ System builtins). No project-owned roles yet.
- A **single** generalized `grants` table (role|permission) rather than two.
- No-escalation enforced in the grant use case (not only via Cedar).
- The infra-local `enum Scope<'a>` query helpers in `postgres/{pipelines,jobs}/
  repository.rs` are unrelated to authz `Scope`; rename later to avoid confusion.
