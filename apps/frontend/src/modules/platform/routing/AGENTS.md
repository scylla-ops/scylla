# `platform/routing` — agent guide

How a module declares itself, and how the shell turns those declarations into a router, a
sidebar and breadcrumbs.

**Layer** `platform/` · alias `@platform/routing`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/routing` — the barrel, never a deep path.
- Imported by all three sides: features declare routes, `core` composes them, `layout` reads the
  breadcrumb contract. It sits in `platform/` because none of those may depend on each other.
- **This is the only module that imports `sv-router`.** Features, `core` and `layout` use the
  functions below. Do not import `sv-router` anywhere else.

## Public API — `index.ts`

```typescript
type ScyllaModule, ModuleRoute, NavEntry, RouteMount, RouteParams, PageLoader
type RouteHandle, Crumb, BreadcrumbParams
type AppRoute, AppRouterConfig, LayoutComponent, RouteWrapper, TrailCrumb
routesFor, navEntriesFor              compose the module declarations
createAppRouter                       starts the router, returns the navigator
routeParams, routePathname            the current route (reactive)
routeTrail, requiredPermission        the handles on the current URL (reactive)
Redirect                              a component that replaces the URL on mount
RouterView                            the component that renders the current route
```

## Layout

```
index.ts                     public API
scylla-module.struct.ts      ScyllaModule, ModuleRoute, NavEntry, RouteMount, RouteParams, PageLoader
route-handle.struct.ts       RouteHandle, Crumb, BreadcrumbParams
app-route.struct.ts          AppRoute, AppRouterConfig, LayoutComponent, RouteWrapper
compose-module-routes.ts     routesFor, navEntriesFor
router-tree.ts               AppRoute tree -> the route object of sv-router
app-router.ts                createAppRouter: the router and its navigator
route-state.ts               reactive accessors on the current route
resolve-target.ts            relative navigation targets (`..`, `members`)
RoutePage.svelte             the page slot of every route: page transition + RouteEntry
RouteEntry.svelte            wrappers, permission guard, page loader, route params as props
Redirect.svelte              replaces the URL on mount
```

## The router library: `sv-router`

The plan in `refacto_svelte.md` §4.2 wanted a router written in the project. The team uses
`sv-router` instead. The adapter is in this module, so a change of router library changes this
module only.

The adapter uses `sv-router` for path matching, history and link clicks. It does not use the
nested layouts, the lazy loading or the hooks of `sv-router`. Every leaf route renders
`RoutePage`, and `RoutePage` reads what it needs from the route metadata (`meta`):

| `meta` field | Set from | Read by |
|---|---|---|
| `page` | `lazy` (or an eager `component`) | `RouteEntry` loads and renders it |
| `redirect` | `redirect` | `RouteEntry` renders `Redirect` |
| `trail` | every `handle` on the path, with its depth | breadcrumbs, route guard |
| `wrappers` | every `wrapper` on the path | `RouteEntry` renders them around the page |
| `shell` | route is in `routes`, not in `publicRoutes` | `RoutePage` animates the page |

Rules that follow from this design:

- **`RouteEntry` reads the route once, when it mounts.** The page transition keeps the old page
  on screen for its exit animation. If the old page read the router reactively, it would show
  the new page during its exit. `RoutePage` mounts a new `RouteEntry` for each pathname.
- **A change of pathname mounts the page again.** A change of the query string does not.
- **Route parameters arrive as props**, on the page and on each wrapper. A page declares the
  parameters it reads: `let { projectId }: { projectId?: string } = $props();`.
- **Public routes and the fallback break out of the shell layout.** `router-tree.ts` writes
  their keys with the `(segment)` syntax of `sv-router`.
- **`sv-router` puts a `/` key first at each level.** That is why the shell is the root layout
  and not a `/` layout group: a `/` group would match `/login` as an organization slug.
- `sv-router` needs `IntersectionObserver`. `src/test/setup.ts` stubs it. Call
  `createAppRouter` inside a test, not at module scope, so the stub is present.

## `RouteMount` — where routes graft

| Mount | Under |
|---|---|
| `public` | outside the auth guard, e.g. `/login` |
| `organization` | `/:organizationSlug` |
| `projects` | `/:organizationSlug/projects` |
| `project` | `/:organizationSlug/projects/:projectId` |

The shell owns the skeleton — auth guard, layout, the org/project sync wrappers. Modules say
which *scope* they belong to instead of restating that nesting. Adding a new mount means
changing the shell, not a module.

## One declaration, three consumers

```
<feature>.module.ts  ──▶  routesFor()      ──▶  router (core/presentation/ui/router/core.router.ts)
                     ──▶  navEntriesFor()  ──▶  sidebar
                     ──▶  handle.breadcrumb ──▶ ScyllaBreadcrumbs
```

`permission` is declared **once** and read by both the route guard and the sidebar — which is
why a link can no longer be visible for a page that will deny you, or hidden for one that would
not. Never gate a page by wrapping it in `RequirePermission` *and* declaring `permission`.

## Things in the composer you will trip over

- **`handle` is static metadata**, set from `permission` + `breadcrumb`. The guard and the
  breadcrumbs read it **without loading the page chunk**. Never put anything in `handle` that
  requires the component.
- **`mergeSharedParents` folds sibling routes claiming the same path segment.** That is how
  `user` owns `users` (the directory) while `organization` owns `users/:userId` (the settings
  page) without either importing the other, and the shared ancestor's breadcrumb applies to
  both. Check for an existing claim before adding a route on a shared segment.
- **The guard takes the deepest match**: a child asking for more than its parent is checked
  against its own requirement. A child that declares nothing gets the permission of its parent.
- `navEntriesFor` sorts by `order` ascending across all modules; ties fall back to registration
  order in `core/di/registry.ts`.
- **A relative navigation starts from the current URL.** `navigateTo('..')` goes to the parent
  page, `navigateTo('members')` goes to a child page. See `resolve-target.ts`.

## `Crumb` — labels vs data

```typescript
{ label: MessageDescriptor, highlight?: string, detail?: MessageDescriptor }
```

`label` and `detail` are translated; `highlight` is business data and stays verbatim in every
locale. They are `` msg`…` `` **descriptors**, which is what lets a module declare routes in a
plain `.ts` file — and they are still translated at render time, so a locale switch updates
them. `BreadcrumbParams` currently offers `projectName`, `organizationName`, `pipelineName`,
`userId`, `jobId`; extend it here if a route needs more.

## Rules that bite here

- **`routes.lazy` is what keeps pages out of the initial chunk. Keep it.** Write it as
  `lazy: () => import('./presentation/ui/X.page.svelte')`. A route without `lazy` is only correct
  for a grouping route that owns a path segment and its children.
- Route declarations belong in `<feature>.module.ts`, never in `core.router.ts`. There is no
  second list to keep in sync — that is the entire point.
- `ScyllaModule` must never be exported from a feature's `index.ts`
  (`module-declaration-is-private`, error): the registry imports it eagerly, and a barrel that
  re-exports UI would pull every page into the entry chunk.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
