# `features/roles` — agent guide

Access-control administration: the role catalog, grants, and the permission vocabulary.

**Layer** `features/` · **id** `roles` · **DI keys** `permissionRepository`, `updateRole`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type RoleEntity, RoleCreationData, GrantEntity
roleConfers
useRoles
useGrants, useScopedGrants, GRANTS_QUERY_KEY
useGrantableRoles
usePermissionLabels
usePermissionSync                     ← mounted once by the shell
humanizeRoleId
```

Never add: `roles.module.ts`, `use-roles-domain.ts`, pages.

## ⚠ The authz split — read this first

`Permission`, `useCan`, `Can`, `RequirePermission` are **not** here. They live in
[`@platform/authz`](../../platform/authz/AGENTS.md), *below* the features, so that gating a
button never means depending on the module that administers roles.

The division:

| Concern | Where | Why |
|---|---|---|
| Asking "may I?" | `@platform/authz` — reads a store, no I/O | any feature may depend on it |
| Loading the answer | **here** — `usePermissionSync` | needs a repository call |
| Administering roles/grants | **here** | it is a feature |

`usePermissionSync` fetches `getMyPermissions()` and fills `usePermissionsStore`. The shell
mounts it **once**. Never call it from a feature, and never import `@platform/authz`'s store to
write to it from elsewhere.

## Data contract

`PermissionRepository` — `domain/repository/permission.repository.ts`:

| Group | Methods |
|---|---|
| Roles | `listRoles`, `getRole`, `createRole`, `updateRole`, `deleteRole` |
| Effective permissions | `getEffectivePermissions(...)`, `getMyPermissions()` |
| Grants | `listGrants(scope?, scopeId?)`, `createGrant(CreateGrantInput)`, `revokeGrant(id)`, `revokeAllAccess(RevokeAllAccessInput)` |
| Vocabulary | `listGrantableRoles(scope?)`, `listPermissionVocabulary()` |

Input types (`CreateGrantInput`, `RevokeAllAccessInput`) live beside the interface, in domain.
Reach it with `useRolesDomain()` **inside a hook only**.

## `UpdateRoleUseCase` — the codebase's only use case

`domain/use-cases/update-role.use-case.ts`, injected as `domain.updateRole`. It survives because
it orchestrates: **read** `getRole` → **apply** the pure `updateRole(role, changes)` entity
function → **save** `updateRole`. A repository method cannot do that.

That is the bar. Anything that would just forward to a repository method is not a use case —
call the repository.

## Layout

```
roles.module.ts                      route + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/role.entity.ts            RoleEntity, RoleCreationData, RoleOrigin,
                                     roleConfers + updateRole (pure)
  entities/grant.entity.ts           GrantEntity
  entities/grantable-role.entity.ts  GrantableRoleEntity
  entities/permission-vocabulary.entity.ts  PermissionVocabularyEntity, PermissionActionEntity
  repository/permission.repository.ts       + CreateGrantInput, RevokeAllAccessInput
  use-cases/update-role.use-case.ts
infrastructure/
  data/grpc-permission-remote.data-source.ts             impl
  repository/data-sources/permission.data-source.ts      interface
  repository/mappers/                grpc-role, grpc-grant, grpc-grantable-role,
                                     grpc-permission, grpc-permission-vocabulary,
                                     grpc-effective-permissions
  repository/default-permission.repository.ts
presentation/
  hooks/use-roles-domain.ts          DI accessor (private)
  hooks/use-roles.ts, use-grants.ts, use-grantable-roles.ts,
  hooks/use-permission-vocabulary.ts, use-permission-labels.ts,
  hooks/use-effective-permissions.ts, use-permission-sync.ts,
  hooks/use-refresh-my-permissions.ts, use-role-assignees.ts,
  hooks/use-grant-target-labels.ts, use-project-grant-eligibility.ts
  ui/Roles.page.tsx, RolesHeader.tsx
  ui/components/RoleListItem.tsx, RoleDetailPanel.tsx
  ui/components/role-detail/         RoleDetailHeader, RoleDetailPermissions,
                                     RoleDetailGrantList, GrantCreator
  ui/components/role-form/           RoleFormDialog, RoleDialogHeader, RoleDialogPermissions
  utils/permission-mapping.ts, permission-tree.ts, role-label.ts
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `roles` | `MANAGE_ROLES` | `RolesPage` |

Sidebar: section **`system`** (not `organization`), order `20`, icon `ShieldIcon`.

## Rules that bite here

- **Role ≠ grant.** A `RoleEntity` is a named bundle of permissions; a `GrantEntity` binds a
  principal to a role at a scope. Deleting a role and revoking a grant are different operations
  with different blast radii — keep them visually and textually distinct.
- **After any grant mutation, refresh the current user's permissions** if they might be the
  subject (`useRefreshMyPermissions`). Otherwise `useCan` answers from a stale store and the UI
  lies about what the user may do.
- `roleConfers` and `updateRole` are **pure entity functions**. Role logic goes in
  `role.entity.ts`, not in a component.
- The permission tree UI is built from the backend's `listPermissionVocabulary()`, not from a
  hardcoded list — `permission-tree.ts` / `permission-mapping.ts` shape it, `CheckboxTree` from
  `@shared` renders it. A new backend permission must appear without a frontend change.
- `humanizeRoleId` and `usePermissionLabels` are the only sanctioned way to render a role or
  permission id to a human. Never interpolate a raw id.
- `useGrantableRoles` is scope-aware: what you may grant on a project differs from an
  organization.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
