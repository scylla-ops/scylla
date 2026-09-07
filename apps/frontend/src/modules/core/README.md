# Core

> [Scylla frontend](../../../README.md) › `app/` › **core** ·
> [agent guide](./AGENTS.md) · [architecture](../../../docs/architecture.md)

The composition root. This is the only part of the application that knows every module exists,
and it is where the pieces are assembled into a running app: the provider stack, the router
skeleton, authentication, and the wrappers that keep the active organization and project in
sync with the URL.

## One list, three derivations

`di/registry.ts` holds the single array of module declarations:

```typescript
export const modules = [LoginModule, DashboardModule, ProjectModule, …];
export const dependencies = Object.fromEntries(modules.map(m => [m.id, m.domain]));
```

From that one array come **all three** of the app's cross-cutting structures:

- the **router**, via `routesFor(modules, mount)`
- the **sidebar**, via `navEntriesFor(modules)`
- the **DI container**, via the `dependencies` map

There is no second list to keep in sync. Registering a feature is one line here plus its own
`<feature>.module.ts` — and registration order sets the order things appear in, so the array
reads roughly top-to-bottom as the app does.

One detail is worth its own sentence: the registry imports each `<feature>.module.ts` **by
path**, never the feature's `index.ts`. Barrels re-export UI, and importing one eagerly here
would drag every page into the initial chunk and undo the lazy routes. It is the only deep
import into a feature allowed anywhere in the codebase, and dependency-cruiser enforces that no
one else does it.

## The router owns the skeleton, not the pages

`Core.router.tsx` describes the shape of the application and nothing about its content:

```
public routes (outside the guard)          ← /login
└── AuthGuard                              ← token or redirect
    └── Layout                             ← sidebar, top bar, breadcrumbs
        ├── /                              → redirect to the user's organization
        └── /:organizationSlug             → sync the org into context
            └── RouteGuard                 → organization-scoped module routes
                └── projects               → project-list module routes
                    └── :projectId         → clean stale context
                        └── RouteGuard     → project-scoped module routes
```

Every leaf is `routesFor(modules, mount)`. Adding a page means editing one module's declaration;
this file does not move. It changes only when a new mount point is introduced or the shell's own
structure does.

`RouteGuard` appears twice — one pathless layout route per scope. It reads the `permission` each
route declared and applies it once, replacing what used to be fifteen hand-written
`RequirePermission` wrappers kept in step with the sidebar by hand. See
[platform/routing](../platform/routing/README.md).

## Context follows the URL

Three small wrappers keep the [context store](../platform/context/README.md) honest:

- **`OrganizationRedirectWrapper`** — landing on `/` sends you to your organization (the one you
  last used, or your first).
- **`OrganizationSyncWrapper`** — resolves the `:organizationSlug` in the URL to a real
  organization and writes it into the store, falling back if the slug does not match anything
  you can see.
- **`ContextCleanerWrapper`** — clears project and pipeline context when it goes stale, and
  redirects out if the `:projectId` in the URL is not a project you have.

They all sync in one direction: **URL → store**. The URL is the source of truth and the store is
its mirror. That is what makes a pasted link work, and it is why there is no wrapper syncing the
other way — two-way sync would need effects on both sides watching each other, which is exactly
the cascade the codebase's React rules exist to prevent.

## Errors are handled once, at the root

`App.tsx` configures the `QueryClient` with a `QueryCache` and a `MutationCache` error handler.
Together they cover every query and every mutation in the app:

- an `UNAUTHENTICATED` response, or a network failure, clears the token and sends the user to
  `/login`;
- anything else is logged and surfaced as a toast built from the error's user-facing message.

**The practical consequence for feature code: do not add your own `onError` toast to a
mutation.** The root handler already shows one, and a second is a duplicate on screen. If a
mutation needs bespoke handling, it needs bespoke *behaviour* — not another toast.

Two acknowledged `//todo`s sit alongside that setup: console noise should probably be narrowed
in production, and domain errors would ideally be toasted by the module that owns them. Both are
deliberate beta-stage trade-offs — verbose global reporting makes user-reported problems easier
to trace.

## The provider stack

```
StrictMode → ThemeProvider → I18nProvider → QueryClientProvider → DependenciesProvider → Router
```

The order matters. `DependenciesProvider` must wrap the router, or every `useModuleDomain` call
throws. `StrictMode` is on, so effects run twice in development — that is a bug detector, not a
nuisance to switch off.

## Related modules

- [layout](../layout/README.md) — the shell UI this router renders into.
- [platform/routing](../platform/routing/README.md) — `routesFor`, `navEntriesFor`, `RouteGuard`.
- [platform/di](../platform/di/README.md) — the provider and accessor this wires up.
- [platform/context](../platform/context/README.md) — the store the wrappers write to.
