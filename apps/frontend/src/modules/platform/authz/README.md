# platform / authz

> [Scylla frontend](../../../../README.md) › `platform/` › **authz** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The authorization primitives: the `Permission` enum, the hooks that answer "may this user do
this?", and the components that gate UI on the answer.

Imported everywhere as `@platform/authz`.

## Why it lives below the features

Almost every feature needs to gate something — a button, a table column, a whole page. If those
primitives lived in [features/roles](../../features/roles/README.md), where roles and grants are
administered, every feature in the app would depend on that one module, and `roles` would end up
depending on itself to gate its own UI.

So authz was split in half along the line of *who needs I/O*:

- **The read side lives here.** `useCan` answers from a Zustand store, synchronously, with no
  network call and no repository. Because it needs nothing, anything may depend on it, and it
  sits in `platform/` below the features.
- **The load side stays in `features/roles`.** Fetching the current user's effective permissions
  needs a repository call, so `usePermissionSync` lives there and the shell mounts it once after
  sign-in.

```
features/roles ──usePermissionSync──▶ usePermissionsStore ──useCan──▶ every feature
   (fetches)                            (platform/authz)              (asks, no deps)
```

The constraint that keeps this honest: **this module must never gain a repository.** The moment
it fetches, it needs a transport and a data contract, and the dependency direction inverts. It
is machine-enforced too — `platform-knows-no-feature` is an error in
`.dependency-cruiser.cjs`.

The practical consequence, worth knowing when you change grants: if a mutation could affect the
*current* user's own access, something has to refresh the store, or `useCan` keeps answering
from stale data.

## The model

- **`Permission`** — a single capability (`LIST_SECRETS`, `MANAGE_ROLES`, …). The enum every
  module gates on.
- **`PermissionScope`** — `org` or `project`. Permissions are held at a scope, and an
  organization-level permission applies inside its projects. Not the reverse.
- **`PrincipalEntity` / `PrincipalKind`** — who holds access: a user, or an app.
- **`EffectivePermissionsEntity`** — the flattened answer for the current user, per scope, after
  all their roles and grants are resolved. `canAccess` is the pure function that queries it.

Scope resolution is the part worth *not* reimplementing. "Does this user have `X` on project
`P`?" means checking the project's own grants and the parent organization's. `useAuthorization`
does it; comparing permission arrays by hand gets it wrong.

## Four ways to gate

| Component / hook | For |
|---|---|
| `<Can permission={…}>` | show a fragment only if allowed |
| `<RequirePermission>` | guard a route or a whole section |
| `<PermissionButton>` | render the action, disabled, when not allowed |
| `useCan` / `useAuthorization` | imperative checks inside a hook |

`PermissionDenied` is the shared denial state, so a blocked page looks the same everywhere.

Prefer `PermissionButton` to hiding an action. A user who cannot see that a capability exists
cannot ask an administrator for it; a disabled button with a reason can.

## Routes gate themselves

You rarely write `<RequirePermission>` around a page by hand. A module declares
`permission` on its route, the composer copies it into the route's `handle`, and `RouteGuard`
from [platform/routing](../routing/README.md) applies it — while the sidebar reads the *same*
declaration to decide whether to show the link.

That single declaration is why a link can no longer be visible for a page that will deny you, or
hidden for one that would not.

## A reminder about what this is for

Client-side gating is user experience, not security. The backend enforces every permission on
every call. This module exists so the UI does not offer actions that will fail — never as the
thing standing between a user and data.

## Related modules

- [features/roles](../../features/roles/README.md) — fills the store; administers roles/grants.
- [platform/routing](../routing/README.md) — `RouteGuard`, which applies route permissions.
- [features/membership](../../features/membership/README.md) — grants as membership.
