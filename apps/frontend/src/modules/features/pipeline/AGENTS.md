# `features/pipeline` — agent guide

Pipelines: their definition, their editor, and the metadata the overviews read.

**Layer** `features/` · **id** `pipeline` · **DI key** `pipelineRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type PipelineEntity
type PipelineMetadata, PipelineStep, PipelineIdentity
PIPELINES_QUERY_KEY, ORGANIZATION_PIPELINES_QUERY_KEY, PIPELINES_QUERY_ROOT
pipelineQueries                // Query options factory (replaces useOrganizationPipelines)
asPipelineFeed
```

Never add: `pipeline.module.ts`, pages — the module registry imports `.module.ts` by path,
not through the barrel.

## Data contract

`PipelineRepository` — `domain/repository/pipeline.repository.ts`:

| Method | Returns |
|---|---|
| `getMetadataByProjectId(...)` | paginated `PipelineMetadata` |
| `getMetadataByOrganizationId(...)` | paginated `PipelineMetadata` |
| `getById(id)` | `PipelineEntity` (full, with steps) |
| `create(pipeline)` | `void` |
| `edit(id, steps, name?)` | `PipelineEntity` |
| `deleteById(id)` | `void` |
| `run(id)` | `void` |

**`PipelineMetadata` vs `PipelineEntity` is the split that matters**: lists fetch metadata (no
steps), the editor fetches the entity. Do not make a list page call `getById`.

Reach the repository with `usePipelineDomain()` **inside a hook only**.

## Layout

```
pipeline.module.ts                           4 routes + DI wiring (private; registry only)
index.ts                                     public API
domain/
  entities/pipeline.entity.ts                PipelineEntity
  structs/pipeline.struct.ts                 PipelineMetadata, PipelineStep, PipelineIdentity
  repository/pipeline.repository.ts
infrastructure/
  repository/data-sources/pipeline-remote.data-source.ts    interface
  data/remote/grpc-pipeline-remote.data-source.ts           impl
  repository/mappers/grpc-pipeline.mapper.ts
  repository/default-pipeline.repository.ts
presentation/
  blueprint.state.svelte.ts                  ViewModel (nodes/edges state) — Runes-based
  pipeline-dashboard.state.svelte.ts        Dashboard ViewModel
  pipeline-script.state.svelte.ts            Script editor state
  pipeline.messages.ts                       i18n message descriptors
  pipeline.queries.ts                        TanStack Query options factories
  pipelines.query-keys.ts                    Query key factories (exported publicly)
  ui/
    *.page.svelte                            Pages (dashboard, creation, update, jobs route)
    editor/                                  PipelineEditor + header, forms, blueprint/
    blueprint/                                Canvas components (BlueprintCanvas, nodes, edges)
    dashboard/                                Dashboard, chart, table, pipeline table columns
  utils/
    blueprint-converter.ts                   Graph ↔ PipelineStep[] (pure, TS only, tested)
    code-mirror.actions.ts                   Svelte action for CodeMirror integration
    create-default-script.ts
    pipeline-script.ts
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `project` | index | `LIST_PIPELINES_BY_PROJECT` | `DashboardPipelinePage` |
| `project` | `create` | `CREATE_PIPELINE` | `PipelineCreationPage` |
| `project` | `edit/:pipelineId` | `UPDATE_PIPELINE` | `PipelineUpdatePage` |
| `project` | `pipelines/:pipelineId/jobs` | `LIST_JOBS_BY_PIPELINE` | `PipelineJobsRoute` |

**No nav entry** — pipelines are the project index, reached by entering a project.

The jobs **list** route is owned here, not by [`jobs`](../jobs/AGENTS.md): the page needs a
**Run** action, which is a pipeline operation. `PipelineJobsRoute` renders `JobsPage` (imported
from `jobs`'s public API) and supplies that action. The job **details** route
(`pipelines/:pipelineId/jobs/:jobId`) is owned by `jobs` itself — it needs nothing from here —
and this module links to it with `scyllaNavigate.goToJobDetails(...)`, never by importing
the page.

## Rules that bite here

- **Query options come from `pipeline.queries.ts`**, not from individual `use-` hooks, and are 
  exported publicly so other modules invalidate what this one reads. After `run()`, also invalidate 
  `JOBS_QUERY_ROOT` / `JOBS_QUERY_KEY(pipelineId)` from `features/jobs` — a run creates a job.
- **CodeMirror is a legitimate use of `$effect`** in `script-editor.svelte.ts` — it's a system 
  outside Svelte that must be synchronized imperatively. That licence does not extend to the rest 
  of the module, which uses Runes.
- **`blueprint.state.svelte.ts` is the graph ViewModel** — it uses Svelte Runes (`$state`, 
  `$effect`) to keep nodes and edges in sync with the document. The canvas takes it as a prop and 
  binds directly to its reactive state.
- **`pipeline-script.state.svelte.ts` holds `script` + `initialScript` only** — draft editor text 
  and the baseline for the dirty check. It is UI state, not server state. Never put the fetched 
  `PipelineEntity` in it.
- `blueprint-converter.ts` is the single translation between the flow graph and `PipelineStep[]`. 
  Both directions live there; do not inline conversion in a component. It's pure TypeScript with 
  full test coverage.
- Largest module in the codebase — respect the ~150-line component rule. New editor UI goes in 
  `ui/editor/blueprint/`, new list UI in `ui/dashboard/pipeline-table/`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
