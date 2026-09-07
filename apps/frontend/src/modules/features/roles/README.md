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

`Permission`, `useCan`, `Can` and `RequirePermission` are **not** in this module. They are in
[platform/authz](../../platform/authz/README.md), one layer below the features.

The reason is dependency direction. Every feature needs to gate a button on a permission. If the
primitives lived here, every feature would depend on the module that administers roles — and
`roles`, which needs to gate its own UI, would depend on itself. Pushing the read side down to
`platform/` breaks that: `useCan` answers from a store, synchronously, with no I/O, so anything
may depend on it.

What stays here is the half that needs a backend call:

```
usePermissionSync  (this module)  ──fills──▶  usePermissionsStore  (platform/authz)
                                                       │
                                                  useCan reads it
                                                       │
                                        ◀── every feature gates on it
```

The shell mounts `usePermissionSync` once, after sign-in. That is the only writer.

The corollary matters when you change grants: if a mutation could affect the *current* user's
own access, the store has to be refreshed, or `useCan` keeps answering from stale data and the
UI shows buttons the server will refuse. `useRefreshMyPermissions` exists for exactly that.

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
is **not** hardcoded — it comes from `listPermissionVocabulary()`, is shaped by
`permission-tree.ts` and `permission-mapping.ts`, and is rendered with `CheckboxTree` from
[shared](../../shared/README.md).

The payoff: when the backend adds a permission, it appears in the editor with no frontend
change. The cost: labels have to be resolved rather than written inline, which is what
`usePermissionLabels` and `humanizeRoleId` are for. Never interpolate a raw permission or role
id into the UI.

## Structure

**Domain** is the richest in the codebase — four entities (role, grant, grantable role,
permission vocabulary), a repository interface with its input types, and the one use case. Role
behaviour lives in `role.entity.ts` as pure functions: `roleConfers` answers whether a role
includes a permission, `updateRole` applies changes immutably.

**Infrastructure** carries six mappers, one per proto shape.

**Presentation** is a master–detail layout: `RoleListItem` on the left, `RoleDetailPanel` on the
right, split into a header, a permissions view and a grant list with `GrantCreator`. The
create/edit dialog lives under `role-form/`.

## Related modules

- [platform/authz](../../platform/authz/README.md) — the read side, and the store this module
  fills.
- [membership](../membership/README.md) — the main consumer: members are derived from grants.
- [user](../user/README.md), [apps](../apps/README.md) — the principals a grant can target.
- [project](../project/README.md), [organization](../organization/README.md) — the two scopes.
