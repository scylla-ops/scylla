# `features/roles` — agent guide

Access-control administration: the role catalog, grants, and the permission vocabulary.

**Layer** `features/` · **id** `roles` · **DI keys** `permissionRepository`, `updateRole` ·
**Svelte** (Phase 4)

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type RoleEntity, RoleCreationData, GrantEntity
roleConfers
humanizeRoleId, scopeLabelOf
roleQueries, roleMutations, grantMutations
refreshMyPermissions, syncMyPermissions, resetPermissionSync
ROLES_QUERY_KEY, GRANTS_QUERY_KEY, GRANTABLE_ROLES_QUERY_KEY
```

Every export is **framework-free** — options factories, pure functions, types. That is load
bearing, not incidental: `membership` (Svelte) and `layout` (still React) both read from here,
and a `queryOptions` object is the only thing `useQuery` and `createQuery` can run against the
same cache entry.

Never add: `roles.module.ts`, a domain accessor, a page, **or any `.svelte` component** — a
component re-exported from a barrel cannot be dropped by Rollup and drags bits-ui into the chunk
of whoever imported the barrel for a type. Export a `load*()` function instead.

## ⚠ The authz split — read this first

`Permission`, `can`, `Can`, `RequirePermission` are **not** here. They live in
[`@platform/authz`](../../platform/authz/AGENTS.md), *below* the features, so that gating a
button never means depending on the module that administers roles.

The division:

| Concern | Where | Why |
|---|---|---|
| Asking "may I?" | `@platform/authz` — reads a store, no I/O | any feature may depend on it |
| Loading the answer | **here** — `syncMyPermissions` | needs a repository call |
| Administering roles/grants | **here** | it is a feature |

`syncMyPermissions(organizationId, projectId)` fetches `getMyPermissions()` and fills
`usePermissionsStore`, **only when the sync key — user + organization + project — changed**. The
guard is in this function, not in its caller, so it survives whichever framework mounts it. The
shell owns *when*: `layout/presentation/hooks/use-permission-sync.ts` is its React binding today
and becomes a Svelte effect in Phase 6, with this function untouched. Call it from **one place**,
never from a feature, and never write the authz store from anywhere else.

`resetPermissionSync()` forgets the key — for sign-out and for tests.

## Data contract

`PermissionRepository` — `domain/repository/permission.repository.ts`:

| Group | Methods |
|---|---|
| Roles | `listRoles`, `getRole`, `createRole`, `updateRole`, `deleteRole` |
| Effective permissions | `getEffectivePermissions(...)`, `getMyPermissions()` |
| Grants | `listGrants(scope?, scopeId?)`, `createGrant(CreateGrantInput)`, `revokeGrant(id)`, `revokeAllAccess(RevokeAllAccessInput)` |
| Vocabulary | `listGrantableRoles(scope?)`, `listPermissionVocabulary()` |

Input types (`CreateGrantInput`, `RevokeAllAccessInput`) live beside the interface, in domain.
Reach it with `getModuleDomain<typeof RolesModule.domain>('roles')` — **inside `roles.queries.ts`
only**, and resolved per call, never at module load, so a test can swap the registry.

## `UpdateRoleUseCase` — the codebase's only use case

`domain/use-cases/update-role.use-case.ts`, injected as `domain.updateRole`. It survives because
it orchestrates: **read** `getRole` → **apply** the pure `updateRole(role, changes)` entity
function → **save** `updateRole`. A repository method cannot do that. `roleMutations.update`
routes through it; `create` and `remove` call the repository, because that is all they are.

That is the bar. Anything that would just forward to a repository method is not a use case.

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
  roles.queries.ts                   every read and write, as plain options objects
  roles-page.state.svelte.ts         the master–detail ViewModel
  role-form.state.svelte.ts          creating/editing one role
  grant-creator.state.svelte.ts      granting one role to one user
  role-assignees.state.svelte.ts     who holds a role, and the revoke
  grant-target-labels.svelte.ts      scope id → human name (fans out for projects)
  grant-eligibility.calculator.ts    pure: who may receive a project grant, and why not
  ui/Roles.page.svelte               the page
  ui/roles.messages.ts               every string the screens show
  ui/components/RoleListItem.svelte, RoleDetailPanel.svelte
  ui/components/role-detail/         RoleDetailHeader, RoleDetailPermissions,
                                     RoleDetailGrantList, GrantCreator, TargetChecklist
  ui/components/role-form/           RoleFormDialog, RoleForm, RoleDialogPermissions,
                                     CheckboxTree, CheckboxTreeNode, checkbox-tree.ts
  utils/permission-mapping.ts, permission-tree.ts, role-label.ts
```

There is no `presentation/hooks/`, no `use-roles-domain.ts` and no Zustand store — the thirteen
hooks became `roles.queries.ts` plus the four ViewModels above.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `roles` | `MANAGE_ROLES` | `Roles.page.svelte` via `sveltePage()` |

Sidebar: section **`system`** (not `organization`), order `20`, icon `ShieldIcon`.

## Rules that bite here

- **Role ≠ grant.** A `RoleEntity` is a named bundle of permissions; a `GrantEntity` binds a
  principal to a role at a scope. Deleting a role and revoking a grant are different operations
  with different blast radii — keep them visually and textually distinct.
- **Every grant mutation already refreshes the caller's own permissions.** `afterGrantChange` in
  `roles.queries.ts` invalidates the whole grant prefix *and* calls `refreshMyPermissions()`.
  Don't do it again at the call site, and don't skip it in a new grant mutation: `can()` would
  keep answering from a stale store and the UI would lie about what the user may do.
- **One grant mutation invalidates every grant list.** A grant created from the project view
  changes the organization's list too, and neither view knows the other exists — hence the shared
  `permission-grants` prefix rather than an exact key.
- `roleConfers` and `updateRole` are **pure entity functions**. Role logic goes in
  `role.entity.ts`, not in a component.
- **The permission tree is data.** It is built from `permission-mapping.ts`'s catalog, shaped by
  `permission-tree.ts`, and rendered by `CheckboxTree`. `checkbox-tree.ts` holds the pure rules —
  a child counts only when its whole parent chain is checked, unchecking a parent clears its
  descendants. Those rules are the permission model's, not a widget's, which is why the tree
  lives here rather than in `shared/`.
- **Implicit permissions are written, never shown.** `IMPLICIT_PERMISSIONS_BY_SCOPE` says what a
  scope confers by construction (`READ_ORGANIZATION` on an organization role) and what rides on a
  stand-in. `withImplicitPermissions` is the only thing allowed to compute the difference —
  `conferredCount` is what the editor displays, and it is the honest number.
- **A permission outside this build's catalog is carried, not deleted.** `preservedCount` in
  `role-form.state.svelte.ts` counts them; they are re-attached on save. An older build silently
  stripping a newer permission is the failure this prevents.
- **A bits-ui checkbox is a `<button>`, and `<Label for>` does not name it.** Every checkbox in
  this module carries an explicit `aria-label` — without it the control announces itself as a
  bare "checkbox" and no test can find it by name.
- `humanizeRoleId` and `permissionLabelOf` are the only sanctioned way to render a role or
  permission id to a human. Never interpolate a raw id.
- `roleQueries.grantable` is scope-aware: what you may grant on a project differs from an
  organization. It needs no permission — it is a compile-time constant on the backend — which
  makes it the only role list a tenant administrator can read.
- **A project grant follows the backend's tenant boundary.** `grant-eligibility.calculator.ts`
  answers *why* a user cannot receive one (`not-admitted` / `cannot-see-projects`) as a value;
  which sentence explains it is the component's business.

## Before done

`pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise && pnpm depcruise:cycles &&
pnpm i18n:collisions` — all clean.
New strings: add them to `presentation/ui/roles.messages.ts` (never inside a `.svelte` —
`lingui extract` does not read one), then `pnpm extract && pnpm compile`.
