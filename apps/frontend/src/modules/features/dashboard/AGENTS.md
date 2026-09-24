# `features/dashboard` — agent guide

The organization landing page: projects, pipelines and recent runs at a glance.

**Layer** `features/` · **id** `dashboard` · **DI key** none (`domain: {}`) · **Svelte** (Phase 4)

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
createOrgOverview, type OrgOverview, type ProjectAccess, type PipelineWithProject
```

Nothing imports it today. It exists so that the day something does, it finds a door — and it
exports **no component**: a `.svelte` re-exported from a barrel cannot be dropped by Rollup and
would drag bits-ui into the chunk of whoever imported the barrel for a type.

## Data contract

**This module owns no data and has no repository.** `domain` is `{}` and it is registered in
`core/di/registry.ts` for its route alone. There is no domain accessor, and adding one would be
wrong.

`createOrgOverview` is a *composition* ViewModel. It reads, through public APIs only:

| From | Read |
|---|---|
| `@platform/context` | `contextStore` — the current organization |
| `@platform/authz` | `can`, `Permission` — per-project access |
| `features/project` | `projectQueries.lookup` |
| `features/pipeline` | `pipelineQueries.byOrganization` + `asPipelineFeed` |
| `features/jobs` | `jobQueries.byOrganization` + `asJobFeed` |
| `features/agents` | `agentQueries.byOrganization`, `agentQueries.statsOf` (in the chart) |

All four reads are `queryOptions` objects, so this module shares one cache entry per resource
with whoever else reads it.

## Layout

```
dashboard.module.ts                     route + nav, empty domain
index.ts                                public API
presentation/
  org-overview.state.svelte.ts          the composition ViewModel
  org-overview.svelte.test.ts
  ui/Dashboard.page.svelte              the page
  ui/RunActivityCard.svelte             recent run activity
  ui/AgentOutcomesChart.svelte          run outcomes over time, in plain SVG
  ui/outcomes-chart.calculator.ts       the chart's geometry — pure, node-tested
  ui/dashboard.messages.ts              every string on the page
```

No `domain/`, no `infrastructure/` — correct for this module, do not add them.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `dashboard` | `READ_ORGANIZATION` | `Dashboard.page.svelte` |

Sidebar: section `organization`, order `10`, icon `LayoutDashboard`.

`READ_ORGANIZATION` is intentionally the same gate as the projects list: the overview is a read
of the organization, and it is where every org-level redirect lands. Do not tighten it to a
narrower permission without moving that redirect target too.

## Rules that bite here

- **Never add a repository.** If the dashboard needs a new figure, add the query to the module
  that *owns* the resource and export the options factory from its `index.ts`. A
  `pipelineRepository` call made from here would fork the query cache into two keys for one
  resource.
- That is why `pipelineQueries.byOrganization` / `jobQueries.byOrganization` live in `pipeline` /
  `jobs` even though the dashboard is their only consumer.
- Cross-feature aggregation belongs in `org-overview.state.svelte.ts`, derived with `$derived` —
  not in a store, not in an effect.
- **The chart's arithmetic stays in `outcomes-chart.calculator.ts`.** It is the half that
  survives a change of framework and the half a test can pin without a DOM
  (`// @vitest-environment node`). The `.svelte` file holds markup, nothing else.
- **The chart's SVG is stretched; its text is not.** `preserveAspectRatio="none"` fills the
  card's width without measuring it, and `vector-effect="non-scaling-stroke"` keeps the stroke
  one pixel under that stretch. Axis ticks, labels and tooltip are HTML *beside* the SVG — a
  `<text>` inside it would be distorted by the same transform.
- **Three messages carry a positional `{0}`** (`Last run {0}`, `over all {0} runs`,
  `over the last {0} of {totalRuns} runs`). The `<Trans>` they came from interpolated an
  expression, and an expression is numbered rather than named; `String(…)` / `Number(…)` in
  `dashboard.messages.ts` is what keeps the msgid — and therefore the French — intact.
  `RunActivityCard.fr.test.ts` is the only thing that would notice if it broke.

## Before done

`pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise && pnpm depcruise:cycles &&
pnpm i18n:collisions` — all clean.
New strings: add them to `dashboard.messages.ts` (never inside a `.svelte`), then
`pnpm extract && pnpm compile`.
