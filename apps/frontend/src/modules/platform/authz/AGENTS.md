# `platform/authz` — agent guide

Authorization primitives: may this user do this?

**Layer** `platform/` · alias `@platform/authz`

## Import rules — the hard one

- **MUST NEVER import a feature.** Enforced by `platform-knows-no-feature` (error). No
  exceptions, no "temporary" ones.
- May import `@shared/*` and other platform capabilities **through their `index.ts` only**
  (`platform-capability-api-only`).
- Features, `core/` and `layout/` import it as `@platform/authz` — the barrel, never a deep
  path.

## Public API — `index.ts`

```typescript
Permission, PermissionScope, PrincipalKind, RoleKind
type AccessEntity, AccessSpec, PrincipalEntity
canAccess
type EffectivePermissionsEntity, EffectiveScopeEntity, PermissionTarget
useAuthorization, useCan
usePermissionsStore
Can, PermissionButton, PermissionDenied, RequirePermission
```

## The split you must not break

**Everything here is read-only and dependency-free.** `useCan` answers from a store,
synchronously. It never calls the backend.

That is precisely what lets authz sit below the features: any feature may gate its UI without
depending on the feature that administers roles.

| Concern | Where |
|---|---|
| Asking "may I?" | **here** — no I/O |
| Loading the answer | [`features/roles`](../../features/roles/AGENTS.md) — `usePermissionSync` |
| Administering roles/grants | `features/roles` |

**Never add a repository, a data source or a query hook to this module.** The moment authz
fetches, it needs a transport and a feature's contract, and the layer inverts.

`usePermissionsStore` has exactly **one writer**: `usePermissionSync`, mounted once by the
shell. If you find yourself writing to it from anywhere else, the fix is upstream.

## Layout

```
index.ts                             public API
domain/
  structs/permission.struct.ts       Permission, PermissionScope, PrincipalKind, RoleKind,
                                     AccessSpec, AccessEntity, PrincipalEntity
  entities/effective-permissions.entity.ts   EffectivePermissionsEntity + canAccess (pure)
presentation/
  hooks/use-authorization.ts         useAuthorization, useCan
  stores/use-permissions.store.ts    the store (single writer: features/roles)
  ui/Can.tsx                         conditional render
  ui/RequirePermission.tsx           route/section gate — used by platform/routing's RouteGuard
  ui/PermissionButton.tsx            button that disables itself
  ui/PermissionDenied.tsx            the denial state
locales/                             this capability has its own catalog
```

## Which gate to use

| Situation | Use |
|---|---|
| Show/hide a fragment | `<Can permission={…}>` |
| Guard a whole route or section | `<RequirePermission>` |
| An action the user can see but not perform | `<PermissionButton>` (visible, disabled) |
| Imperative check in a hook | `useCan(Permission.X)` |
| Several checks / scoped check | `useAuthorization()` |

Prefer `PermissionButton` over hiding an action outright: a user who cannot tell an action
exists cannot ask for access to it.

## Rules that bite here

- **Route permissions are declared on the module, not here.** A `ModuleRoute.permission` flows
  into `RouteHandle` and `RouteGuard` (in [`platform/routing`](../routing/AGENTS.md)) renders
  `RequirePermission` for you. Do not wrap a page by hand as well.
- `Permission` is the enum every module gates on — adding one means adding it here, and the
  role editor picks it up from the backend vocabulary without further changes.
- `canAccess` in `effective-permissions.entity.ts` is **pure**. Authorization logic goes there,
  not into a component.
- Scope matters: a permission held at the organization applies to its projects; one held on a
  project does not apply upward. `useAuthorization` handles this — do not compare permission
  arrays by hand.
- Client-side gating is UX, not security. The backend enforces. Never treat a passing `useCan`
  as proof.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
