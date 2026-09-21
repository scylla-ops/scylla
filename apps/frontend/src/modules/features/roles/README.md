# Roles

> [Scylla frontend](../../../../README.md) › `features/` › **roles** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Access-control administration: the catalog of roles, the grants that bind principals to them,
and the permission vocabulary underneath. The page lives at `/:org/roles`, in the sidebar's
**System** section, behind `MANAGE_ROLES`.

## The vocabulary

Three words that are easy to conflate and mean different things:

- **Permission** — a single capability, like `LIST_SECRETS`. The atom.
- **Role** — a named bundle of permissions (`RoleEntity`). "Project Admin" is a role.
- **Grant** — a binding: *this principal* holds *this role* at *this scope* (`GrantEntity`).

Roles are defined once for the installation; grants are handed out per organization or project.
Deleting a role affects everyone holding it; revoking a grant affects one principal. The UI
keeps them apart deliberately.

## Why the primitives live elsewhere

`Permission`, `can`, `Can` and `RequirePermission` are **not** in this module. They are in
[platform/authz](../../platform/authz/README.md), one layer below the features.

The reason is dependency direction. Every feature needs to gate a button on a permission. If the
primitives lived here, every feature would depend on the module that administers roles — and
`roles`, which needs to gate its own UI, would depend on itself. Pushing the read side down to
`platform/` breaks that: `can()` answers from a store, synchronously, with no I/O, so anything
may depend on it.

What stays here is the half that needs a backend call:

```
syncMyPermissions  (this module)  ──fills──▶  usePermissionsStore  (platform/authz)
                                                       │
                                                   can() reads it
                                                       │
                                        ◀── every feature gates on it
```

`syncMyPermissions` is a plain function with the "has anything actually changed?" guard inside
it — user, organization, project. That guard used to be a React ref inside an effect, which is
why the shell had to be careful about how often it re-rendered; now the shell can call it as
often as it likes and the backend is hit only when the answer could differ. The shell still owns
*when*, and it is the only caller.

The corollary matters when you change grants: if a mutation could affect the *current* user's
own access, the store has to be refreshed, or `can()` keeps answering from stale data and the UI
shows buttons the server will refuse. Every grant mutation in `roles.queries.ts` already does it,
unprompted — that is not something a call site should have to remember.

## The one surviving use case

The codebase once had 65 use cases and 64 were pure forwarding — `execute(args)` calling
`repository.method(args)` under a second name. They were deleted. One remains, and it is here:

```typescript
// UpdateRoleUseCase
getRole(id)                          // read
  .map(role => updateRole(role, changes))   // apply a pure entity function
  .flatMapAsync(updated => repository.updateRole(updated))   // save
```

It earns its place because it *orchestrates*: two repository calls with a pure domain
transformation between them, which no single repository method can express. Use it as the bar
before adding another one anywhere in the codebase.

## The permission tree is data, not code

The role editor shows every permission grouped into a tree of resources and actions. That tree
is **not** hardcoded — it is shaped by `permission-mapping.ts` and `permission-tree.ts`, and
rendered by `CheckboxTree` in this module. It used to live in `shared/`, on the strength of being
"generic"; it never had a second consumer, and the rules it enforces are the permission model's,
so the port brought it home.

The payoff: when the backend adds a permission, it appears in the editor with no frontend
change. The cost: labels have to be resolved rather than written inline, which is what
`permissionLabelOf` and `humanizeRoleId` are for. Never interpolate a raw permission or role id
into the UI.

Some of the tree is deliberately *not* shown. An organization role always confers
`READ_ORGANIZATION` — there is no membership table on the backend, so belonging to an
organization **is** holding a grant at its scope, and offering it as a checkbox would only let
someone build a role that admits a person to a place they cannot see. Others ride on a stand-in:
ticking "list the projects of the organization" writes `READ_PROJECT` too, because the backend
splits across two RPCs what is one capability to a human. The editor writes all of it and shows
a single honest count; `withImplicitPermissions` is the one place that knows the difference.

## Structure

**Domain** is the richest in the codebase — four entities (role, grant, grantable role,
permission vocabulary), a repository interface with its input types, and the one use case. Role
behaviour lives in `role.entity.ts` as pure functions: `roleConfers` answers whether a role
includes a permission, `updateRole` applies changes immutably.

**Infrastructure** carries six mappers, one per proto shape.

**Presentation** is Svelte since Phase 4, and the shape of it is the part worth knowing. The
thirteen React hooks collapsed into two kinds of file: `roles.queries.ts`, which declares every
read and write as a plain options object with no framework in it, and four
`*.state.svelte.ts` ViewModels, one per view rather than one per module. The queries file is what
lets `membership` and the still-React shell read the same role catalog out of the same cache
entry; the ViewModels are where filters, selection and mutations are orchestrated, so the
components stay about rendering.

Two pieces stayed pure TypeScript on purpose, because they are rules rather than UI:
`grant-eligibility.calculator.ts` (why a user may not receive a project grant) and
`checkbox-tree.ts` (a child counts only when its whole parent chain is checked). Both are tested
without a DOM, and both would survive another change of framework.

The screen itself is a master–detail layout: `RoleListItem` on the left, `RoleDetailPanel` on the
right, split into a header, a permissions view and a grant list with `GrantCreator`. The
create/edit dialog lives under `role-form/`, and its reset is `{#key open}` — reopening the
dialog builds a new form seeded from the role at hand, where React needed an effect on
`[open, role]` and a frame showing the previous role's values.

## Related modules

- [platform/authz](../../platform/authz/README.md) — the read side, and the store this module
  fills.
- [membership](../membership/README.md) — the main consumer: members are derived from grants.
- [user](../user/README.md), [apps](../apps/README.md) — the principals a grant can target.
- [project](../project/README.md), [organization](../organization/README.md) — the two scopes.
