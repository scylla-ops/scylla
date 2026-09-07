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
enforces that no *other* module does the same.

## Layout

```
di/registry.ts                       THE module list + the DI map
presentation/ui/App.tsx              provider stack, QueryClient, global error handling
presentation/ui/router/
  Core.router.tsx                    the shell skeleton
  Auth.guard.tsx                     token present? → Outlet, else /login
  OrganizationSync.wrapper.tsx       URL slug → context store
  OrganizationRedirect.wrapper.tsx   `/` → the user's organization
  ContextCleaner.wrapper.tsx         drops stale project/pipeline context
locales/                             the shell's own catalog
```

## `di/registry.ts` — the only list

```typescript
export const modules = [LoginModule, DashboardModule, …] as const satisfies readonly ScyllaModule[];
export const dependencies: DomainRegistry = Object.fromEntries(
  modules.map(module => [module.id, module.domain]),
);
```

Routes, sidebar entries **and** DI are all derived from this one array. **Registration order
decides sidebar and route order within a section** (after `NavEntry.order`), so the list reads
roughly top-to-bottom as the app does.

Adding a feature = add its `*.module.ts` here. There is no second list.

## `Core.router.tsx` — the skeleton, and only the skeleton

```
routesFor(modules, 'public')                     ← outside the guard
└── AuthGuard
    └── Layout (navEntries)
        ├── index → OrganizationRedirectWrapper
        └── /:organizationSlug → OrganizationSyncWrapper
            └── RouteGuard → routesFor(modules, 'organization')
                └── projects → routesFor(modules, 'projects')
                    └── :projectId → ContextCleanerWrapper
                        └── RouteGuard → routesFor(modules, 'project')
```

**Never add a page here.** Adding a page is a change to one module's `*.module.ts`; this file
does not move. It changes only when a *mount point* is added or the shell's structure changes.

## Global error handling lives in `App.tsx`

The `QueryClient` carries a `QueryCache` **and** a `MutationCache` `onError`:

- `UNAUTHENTICATED` or a network error → clear `localStorage.token`, hard-redirect to `/login`.
- Otherwise → `error.log()` and `toast.error(error.userMessage())`.

**Consequence: individual hooks must NOT add their own `onError` toast** — you get a double
toast. This is the codebase's documented rule for mutations.

Two `//todo`s are recorded there deliberately (production console noise; domain errors ideally
toasted by the owning module). Leave them unless you are addressing them.

## Rules that bite here

- `localStorage.token` is a three-way contract: `features/login` writes it, `platform/grpc`
  reads it for the auth header, `Auth.guard.tsx` reads it to redirect. Change all three at once.
- The **wrappers own the URL → store direction** of context sync. Their `useEffect`s are
  legitimate (URL is an outside-React system). Do not add a store → URL sync — the URL is the
  source of truth.
- `ContextCleanerWrapper` also handles a project id that no longer exists (redirects out).
- Provider order in `App.tsx` matters: `ThemeProvider` → `I18nProvider` → `QueryClientProvider`
  → `DependenciesProvider` → router. `DependenciesProvider` must wrap the router, or every
  `useModuleDomain` throws.
- `StrictMode` is on: effects run twice in dev. Fix the effect, do not remove `StrictMode`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
