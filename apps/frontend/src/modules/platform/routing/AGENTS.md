# `platform/routing` — agent guide

How a module declares its routes, and the router that the shell builds from these
declarations. The router, the sidebar and the breadcrumbs all come from the declarations.

**Layer** `platform/` · alias `@platform/routing` · **no router library**

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/routing` — the barrel, never a deep path.
- Imported by three sides: features declare routes, `core` builds the router, `layout` reads
  the sidebar and breadcrumb contracts. It is in `platform/` because none of those three may
  depend on the others.
- Navigation goes through `navigateTo` / `scyllaNavigate` of `@platform/context`, never through
  this module.

## Public API — `index.ts`

```typescript
type ScyllaModule, ModuleRoutes, ModuleRoute, NavLink, RouteMount, RouteSource
type PageLoader, PageComponent, RouteParams
type Crumb, BreadcrumbFn, BreadcrumbParams
type AppRouterConfig, MountDefinition, LayoutComponent, RouteWrapper
compileRoutes, type CompiledRoute, type RouteTable   declarations -> the flat route table
navEntriesFor, type NavEntry                         declarations -> the sidebar entries
createAppRouter                                      compiles the routes, returns the navigator
routeParams, routePathname                           the current route (reactive)
routeTrail, type TrailCrumb                          the crumbs of the current URL (reactive)
RouterView                                           renders the current route
Redirect                                             replaces the URL on mount
```

## Layout

```
index.ts                               public API
declaration/                           what a module and the shell write
  scylla-module.struct.ts              ScyllaModule, ModuleRoute, NavLink, RouteMount, PageLoader
  crumb.struct.ts                      Crumb, BreadcrumbFn, BreadcrumbParams
  app-router-config.struct.ts          AppRouterConfig, MountDefinition, LayoutComponent, RouteWrapper
compilation/                           declarations -> data, pure functions (node tests)
  route-path.ts                        path segments: split, join, key, prefix, specificity
  flatten-routes.ts                    flattenModuleRoutes = unfoldRoutes + mergeSamePath
  compile-routes.ts                    compileRoutes: the flat, sorted route table
  nav-entries.ts                       navEntriesFor
runtime/                               the router in the browser
  location.svelte.ts                   the reactive URL, history, link clicks
  match-route.ts                       matchRoute: URL -> route + params
  route-state.svelte.ts                the current match and its accessors
  resolve-target.ts                    relative navigation targets (`..`, `members`)
  app-router.ts                        createAppRouter
view/                                  the components
  RouterView/                          layout of the mount, or the fallback
    RouterView.svelte
    RouterView.test.ts + fixtures/     the router, end to end
  RoutePage.svelte                     page transition, one RouteEntry per pathname
  RouteEntry.svelte                    wrappers, permission guard, page loader, params as props
  Redirect.svelte
```

The tests of `compilation/` and `runtime/` are in their `__test__/` folder, and run in the
`node` environment except the ones that need the DOM.

## Declaring routes

```typescript
routes: {
  organization: [
    {
      path: 'agents',
      permission: Permission.LIST_AGENTS,
      breadcrumb: () => ({ label: msg`Agents` }),
      page: () => import('./presentation/ui/Agents.page.svelte'),
      nav: { section: 'organization', title: msg`Agents`, icon: HardDriveIcon, order: 40 },
      children: [
        { path: ':agentId', permission: Permission.READ_APP, breadcrumb: …, page: … },
      ],
    },
  ],
},
```

| Field        | Meaning                                                                                                                      |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `path`       | Relative to the parent, or to the mount. Can hold several segments. A child without `path` is the page of its parent's path. |
| `page`       | `() => import('./X.page.svelte')`. **Keep it lazy**: it keeps the page out of the entry chunk.                               |
| `redirect`   | A target. A relative target starts from the URL of the route.                                                                |
| `permission` | Guards **this page only**. There is no inheritance: a child declares its own.                                                |
| `breadcrumb` | The crumb of this path. It shows on this page and on each page below this path.                                              |
| `nav`        | A sidebar link. It takes its URL and its `permission` from the route. `organization` only.                                   |
| `children`   | Routes below this path.                                                                                                      |

## `RouteMount` — where routes graft

| Mount          | Path                                     | What the shell adds                                      |
| -------------- | ---------------------------------------- | -------------------------------------------------------- |
| `public`       | `/`                                      | nothing: no layout, no auth guard — e.g. `/login`        |
| `app`          | `/`                                      | `AppShell` (auth guard + layout). Only the shell uses it |
| `organization` | `/:organizationSlug`                     | `OrganizationSync` wrapper                               |
| `project`      | `/:organizationSlug/projects/:projectId` | `ContextCleaner` wrapper, "Project" crumb                |

The mounts are data, in `core/presentation/ui/router/core.router.ts`. A new mount changes the
`RouteMount` union and that file, never a feature.

## The compilation

`compileRoutes` turns the declarations into a flat table. The steps have names; read them in
this order:

1. **`unfoldRoutes`** — each tree becomes a list. A child gets the path of its parents.
2. **`mergeSamePath`** — the routes on the same mount and path become one route. Parameter
   names do not count (`:id` = `:userId`). This is how two modules share a path without an
   import: `user` declares `users`, `organization` declares `users/:userId`.
3. **`placeInMount`** — each route gets the path of its mount in front of its own.
4. **`trailOf`** — the crumbs of a page are the crumbs of each path that its path starts with.
   So the page of `pipelines/:pipelineId/jobs/:jobId` (module `jobs`) shows the "Jobs" crumb
   of `pipelines/:pipelineId/jobs` (module `pipeline`).
5. **`bySpecificity`** — a static segment before a parameter, so `/login` matches before
   `/:organizationSlug`.

Only a route with a `page` or a `redirect` goes into the table. The others give crumbs only.

**The compilation throws** at startup, and in `core/di/module-permissions.test.ts`, when:

- two declarations of one path set the same field (`page`, `permission`, `breadcrumb`, …);
- a route has a `page` and a `redirect`;
- a route has a `permission` or a `nav` but no `page`;
- two mounts put a page on the same URL;
- a `nav` is on a route that is not in `organization`.

## The runtime

- `location.svelte.ts` holds the URL in `$state`. `changeLocation` calls
  `history.pushState` / `replaceState`. `listenToLocation` (started by `RouterView`) follows
  `popstate` and the clicks on a link of the app: left button, no modifier key, same origin, no
  `target`, no `download`. A link to a hash of the same page stays with the browser.
- `matchRoute` returns the first route of the table whose segments fit the URL.
- `createAppRouter` installs the table and returns the `AppNavigator` of `@platform/context`.

## Rules that follow from the design

- **`RouteEntry` reads the route once, when it mounts.** The page transition keeps the old page
  on screen for its exit animation. If the old page read the route reactively, it would show
  the new page during its exit. `RoutePage` mounts a new `RouteEntry` for each pathname.
- **A change of pathname mounts the page again, with its wrappers.** A change of the query string
  does not.
- **Route parameters arrive as props**, on the page and on each wrapper (`params`). A page
  declares the parameters it reads: `let { projectId }: { projectId?: string } = $props();`.
- **A route in a mount with a `layout` animates.** The others (`public`, the fallback) render
  alone.
- **`permission` is declared once.** The guard and the sidebar read the same value. Never also
  wrap a page in `RequirePermission`.
- `handle`, `index`, `lazy` and `mount` on a route do not exist any more. Use `page`, a child
  without `path`, and the mount key.

## `Crumb` — words and data

```typescript
{ label: MessageDescriptor, highlight?: string, detail?: MessageDescriptor }
```

`label` and `detail` are translated; `highlight` is business data and stays verbatim in every
locale. They are `` msg`…` `` descriptors, so a module declares its routes in a plain `.ts`
file, and a locale switch still updates them. `BreadcrumbParams` offers `projectName`,
`organizationName`, `pipelineName`, `userId`, `jobId`; add a field here if a route needs more.

## Before done

`pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
