# `core` — agent guide

The composition root: the one place that knows every module.

**Layer** `app/` (top) · alias `@core/*`

## Import rules

- **May import anything** — features, platform, shared. It is the top of the graph.
- Features and `layout/` **must reach features through their `index.ts`**
  (`shell-uses-feature-api`, error) — with the one exception below.
- Nothing may import `core/`. A feature importing the shell is an error
  (`no-feature-imports-the-shell`).

## The one deep import that is correct

`core/di/registry.ts` imports each `<feature>/<feature>.module.ts` **directly by path**, never
`<feature>/index.ts`.

That is the only sanctioned door. The barrels re-export UI, and importing one here would pull
every page into the initial chunk and undo the lazy routes. `module-declaration-is-private`
enforces that no _other_ module does the same.

## Layout

```
di/registry.ts                          THE module list + the DI map
di/module-permissions.test.ts           conformance: every page declares a gate
di/feature-permissions.test.ts          conformance: features gate, shared queries check
presentation/ui/App.svelte              theme toggle, router view, toaster
presentation/ui/ThemeToggle.svelte
presentation/ui/router/
  core.router.ts                        the shell skeleton (appRoutes) + startRouter()
  core.messages.ts                      the shell's own messages
  AppShell.svelte                       root layout: AuthGuard + layout's Layout
  Auth.guard.svelte                     token present? → routes, else /login
  OrganizationSync.wrapper.svelte       URL slug → context store (organization-sync.svelte.ts)
  OrganizationRedirect.wrapper.svelte   `/` → the user's organization (organization-redirect.svelte.ts)
  ContextCleaner.wrapper.svelte         drops stale project/pipeline context (context-cleaner.svelte.ts)
  LoginRedirect.svelte                  the fallback for a URL that no route matches
locales/                                the shell's own catalog
src/main.ts                             entry: loads the locale, starts the router, mounts App
```

## `di/registry.ts` — the only list

```typescript
export const modules = [LoginModule, DashboardModule, …] as const satisfies readonly ScyllaModule[];
export const dependencies: DomainRegistry = Object.fromEntries(
  modules.map(module => [module.id, module.domain]),
);
setDependencyRegistry(dependencies);
```

Routes, sidebar entries **and** DI are all derived from this one array. **Registration order
decides sidebar and route order within a section** (after `NavEntry.order`), so the list reads
roughly top-to-bottom as the app does.

Adding a feature = add its `*.module.ts` here. There is no second list.

## `di/module-permissions.test.ts` — the gate that reads that list

Because everything is derived from `modules`, one test can hold the whole app to a rule instead
of thirteen. It enumerates the _compiled_ routes (`compileRoutes(appRoutes)`) and asserts:

**Every page behind `AuthGuard` declares a `permission`** — on the page itself. There is no
inheritance from a parent route. A route with no `page` only gives a crumb and is not reported.

`UNGATED_PAGES` is keyed by the full path (`/:organizationSlug/marketplace`). It is a
**ratchet** — entries may be removed, never added without a real reason, and a stale entry fails
the suite too. `public` routes are exempt structurally.

A sidebar link needs no check: `nav` is part of its route and takes the route's permission.

## `di/feature-permissions.test.ts` — the same idea, one level down

It reads source, enumerated from `modules`:

1. **A feature that mutates gates something in its UI.**
2. **A query another feature imports checks for itself.** Crossing a barrel means running outside
   the owner's route guard.

Both are **completeness, never correctness**. `UNGATED_FEATURES` and `UNCHECKED_SHARED_HOOKS`
are ratchets. Entries marked `SEEDED DEBT` or `TRIAGE` are open questions, not decisions.

## `core.router.ts` — the mounts, and only the mounts

```
mounts
  public         /                                        no layout, no guard
  app            /                                        layout AppShell (AuthGuard + Layout)
  organization   /:organizationSlug                       wrapper OrganizationSyncWrapper
  project        /:organizationSlug/projects/:projectId   wrapper ContextCleanerWrapper, crumb "Project"
modules: [shellRoutes, ...modules]
  shellRoutes    app: /                   -> OrganizationRedirectWrapper
                 organization: (its root) -> redirect 'dashboard'
fallback: LoginRedirect
```

**Never add a page here.** Adding a page is a change to one module's `*.module.ts`; this file
does not move. It changes only when a _mount_ is added or the shell's structure changes.

The route guard is not here. Every page goes through `RouteEntry` in `@platform/routing`, which
applies the `permission` of the page.

`startRouter()` creates the router and installs it with `setAppNavigator`. `main.ts` calls it
once, before it mounts `App`.

## The wrappers

A wrapper is a `RouteWrapper` (`@platform/routing`): it gets the route parameters as the
`params` prop, and renders `children`. The logic of each wrapper is in its `*.svelte.ts` file,
and the tests run that file without a component.

- The wrappers own the **URL → store** direction of context sync. The URL is the source of
  truth. Do not add a store → URL sync.
- **A wrapper reads its parameters once, when it mounts.** The router mounts the wrappers again
  for each new pathname. During the exit animation, the old wrappers stay on screen, so they
  must not react to the new URL.
- The effects read the context store with `untrack`. If an effect depended on the active
  organization, the old wrapper and the new wrapper could set the store one after the other.
- `ContextCleanerWrapper` also handles a project id that no longer exists (redirects out).

## Global error handling lives in `@platform/query`

The `QueryClient` carries a `QueryCache` **and** a `MutationCache` `onError`:

- `UNAUTHENTICATED` or a network error → clear `localStorage.token`, hard-redirect to `/login`.
- Otherwise → `error.log()` and `toast.error(error.userMessage())`.

**Consequence: individual queries and mutations must NOT add their own `onError` toast** — you
get a double toast.

## Rules that bite here

- `localStorage.token` is a three-way contract: `features/login` writes it, `platform/grpc`
  reads it for the auth header, `Auth.guard.svelte` reads it to redirect. Change all three at
  once.
- There are no providers. The query client, the DI registry, the i18n instance and the stores
  are module singletons. `App.svelte` only renders the theme toggle, the router view and the
  toaster.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
