# `platform/routing` — agent guide

How a module declares itself, and how the shell turns those declarations into a router, a
sidebar and breadcrumbs.

**Layer** `platform/` · alias `@platform/routing`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/routing` — the barrel, never a deep path.
- Imported by all three sides: features declare routes, `core` composes them, `layout` reads the
  breadcrumb contract. It sits in `platform/` because none of those may depend on each other.

## Public API — `index.ts`

```typescript
type ScyllaModule, ModuleRoute, NavEntry, RouteMount
type RouteHandle, Crumb, BreadcrumbParams
routesFor, navEntriesFor
RouteGuard
```

## Layout

```
index.ts                             public API
scylla-module.struct.ts              ScyllaModule, ModuleRoute, NavEntry, RouteMount
route-handle.struct.ts               RouteHandle, Crumb, BreadcrumbParams
compose-module-routes.ts             routesFor, navEntriesFor
RouteGuard.tsx                       applies a route's declared permission
```

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
<feature>.module.ts  ──▶  routesFor()      ──▶  react-router
                     ──▶  navEntriesFor()  ──▶  sidebar
                     ──▶  handle.breadcrumb ──▶ ScyllaBreadcrumbs
```

`permission` is declared **once** and read by both `RouteGuard` and the sidebar — which is why
a link can no longer be visible for a page that will deny you, or hidden for one that would not.
Never gate a page by wrapping it in `RequirePermission` *and* declaring `permission`.

## Things in the composer you will trip over

- **`handle` is static metadata**, set from `permission` + `breadcrumb`. It works alongside
  `lazy`: the guard and breadcrumbs read it **without waiting for the chunk to load**. Never put
  anything in `handle` that requires the component.
- **`mergeSharedParents` folds sibling routes claiming the same path segment.** That is how
  `user` owns `users` (the directory) while `organization` owns `users/:userId` (the settings
  page) without either importing the other, and the shared ancestor's breadcrumb applies to
  both. Check for an existing claim before adding a route on a shared segment.
- **`RouteGuard` takes the deepest match**: a child asking for more than its parent is checked
  against its own requirement. One pathless layout route per mount replaces fifteen hand-written
  wrappers.
- `navEntriesFor` sorts by `order` ascending across all modules; ties fall back to registration
  order in `core/di/registry.ts`.

## `Crumb` — labels vs data

```typescript
{ label: MessageDescriptor, highlight?: string, detail?: MessageDescriptor }
```

`label` and `detail` are translated; `highlight` is business data and stays verbatim in every
locale. They are `` msg`…` `` **descriptors, not JSX**, which is what lets a module declare
routes in a plain `.ts` file — and they are still translated at render time, so a locale switch
updates them. `BreadcrumbParams` currently offers `projectName`, `organizationName`,
`pipelineName`, `userId`; extend it here if a route needs more.

## Rules that bite here

- **`routes.lazy` is what keeps pages out of the initial chunk. Keep it.** A route without
  `lazy` is only correct for a pure grouping route that owns a path segment and its children
  (react-router renders an `Outlet`).
- Route declarations belong in `<feature>.module.ts`, never in `Core.router.tsx`. There is no
  second list to keep in sync — that is the entire point.
- `ScyllaModule` must never be exported from a feature's `index.ts`
  (`module-declaration-is-private`, error): the registry imports it eagerly, and a barrel that
  re-exports UI would pull every page into the entry chunk.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
