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
di/module-permissions.test.ts        conformance: every page declares a gate
di/feature-permissions.test.ts       conformance: features gate, shared hooks check
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

## `di/module-permissions.test.ts` — the gate that reads that list

Because everything is derived from `modules`, one test can hold the whole app to a rule instead
of thirteen. It enumerates the *composed* route trees (`routesFor`) and asserts:

1. **Every page behind `AuthGuard` declares a `permission`** — on itself or on an ancestor,
   matching `RouteGuard`'s deepest-match rule. A route with no `lazy` is a grouping node and is
   walked through, not reported.
2. **A sidebar link and the page it opens require the same permission.** `permission` is written
   twice — once in `routes`, once in `nav` — and nothing but this test stops the two drifting.
   It also fails on a nav `url` that no route renders.

The point is the *default*: a page added tomorrow is checked the day its module joins the
registry, without anyone remembering to write a test for it. Per-component tests pin gates that
exist; this one fails for gates that don't.

`UNGATED_PAGES` is a **ratchet** — entries may be removed, never added without a real reason,
and a stale entry fails the suite too. `mount: 'public'` is exempt structurally.

A new mount in `Core.router.tsx` means adding it to `GUARDED_MOUNTS` (and `SHELL_SEGMENTS`, if
the shell owns the segment a nav entry addresses — `projects` is the one such case today).

## `di/feature-permissions.test.ts` — the same idea, one level down

Route declarations are typed, so the rules above can walk them. The gating a feature applies to
its own *buttons* is not declared anywhere, so this file reads source instead. Two rules,
enumerated from `modules` so a new feature is checked on arrival:

1. **A feature that mutates gates something in its UI.** If nothing under its `presentation/ui/`
   ever mentions a `Permission`, every write it offers is open to anyone who reaches the page.
2. **A query hook another feature imports checks for itself.** Crossing a barrel means running
   outside the owner's route guard — the consumer's page was entered on the *consumer's*
   permission. `useJobsByPipelines` is the one that already does this.

Both are **completeness, never correctness**: they catch a feature with no gating and a hook that
trusts its caller; they cannot catch the wrong permission on the right button. Doing that would
need the mutation→permission mapping declared somewhere — today it is spread across a hook, a
table, a child component and a route. Until then, correctness lives in the per-component tests.

`UNGATED_FEATURES` and `UNCHECKED_SHARED_HOOKS` are ratchets, seeded with today's state and
shrinking only. Entries marked `SEEDED DEBT` or `TRIAGE` are open questions, not decisions.

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

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
