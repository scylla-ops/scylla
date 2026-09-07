# `features/dashboard` — agent guide

The organization landing page: projects, pipelines and recent runs at a glance.

**Layer** `features/` · **id** `dashboard` · **DI key** none (`domain: {}`)

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
useOrgOverview, type ProjectAccess
```

## Data contract

**This module owns no data and has no repository.** `domain` is `{}` and it is registered in
`core/di/registry.ts` for its route alone. There is no `use-dashboard-domain.ts`, and adding
one would be wrong.

`useOrgOverview` is a *composition* hook. It reads, through public APIs only:

| From | Hook |
|---|---|
| `@platform/context` | `useContextStore` — the current organization |
| `@platform/authz` | `useAuthorization`, `Permission` — per-project access |
| `features/project` | `useOrganizationProjects` |
| `features/pipeline` | `useOrganizationPipelines` |
| `features/jobs` | `useOrganizationJobs` |

## Layout

```
dashboard.module.ts                  route + nav, empty domain
index.ts                             public API
presentation/
  hooks/use-org-overview.ts          the composition hook
  ui/Dashboard.page.tsx              the page
  ui/AgentOutcomesChart.tsx          run outcomes over time
  ui/RunActivityCard.tsx             recent run activity
```

No `domain/`, no `infrastructure/` — correct for this module, do not add them.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `dashboard` | `READ_ORGANIZATION` | `DashboardPage` |

Sidebar: section `organization`, order `10`, icon `LayoutDashboard`.

`READ_ORGANIZATION` is intentionally the same gate as the projects list: the overview is a read
of the organization, and it is where every org-level redirect lands. Do not tighten it to a
narrower permission without moving that redirect target too.

## Rules that bite here

- **Never add a repository.** If the dashboard needs a new figure, add the query to the module
  that *owns* the resource and export the hook from its `index.ts`. A `pipelineRepository` call
  made from here would fork the query cache into two keys for one resource.
- That is why `useOrganizationPipelines` / `useOrganizationJobs` live in `pipeline` / `jobs`
  even though the dashboard is their only consumer.
- Cross-feature aggregation belongs in `use-org-overview.ts`, derived during render — not in a
  store, not in an effect.
- Use `useQueries` for per-project or per-pipeline fan-out; never loop `useQuery`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
