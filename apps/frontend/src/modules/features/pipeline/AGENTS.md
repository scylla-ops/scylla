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
useOrganizationPipelines
```

Never add: `pipeline.module.ts`, `use-pipeline-domain.ts`, pages.

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
pipeline.module.ts                   4 routes + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/pipeline.entity.ts        PipelineEntity
  structs/pipeline.struct.ts         PipelineMetadata, PipelineStep, PipelineIdentity
  repository/pipeline.repository.ts
infrastructure/
  repository/data-sources/pipeline-remote.data-source.ts    interface
  data/remote/grpc-pipeline-remote.data-source.ts           impl
  repository/mappers/grpc-pipeline.mapper.ts
  repository/default-pipeline.repository.ts
presentation/
  hooks/pipelines.query-keys.ts      key factories — import these, never inline a key
  hooks/use-pipeline-domain.ts       DI accessor (private)
  hooks/use-pipeline.ts, use-pipelines-metadata.ts, use-organization-pipelines.ts
  hooks/use-create-pipeline.ts, use-update-pipeline.ts, use-delete-pipeline.ts,
  hooks/use-duplicate-pipeline.ts, use-run-pipeline.ts
  hooks/use-blueprint-state.ts       ReactFlow nodes/edges state
  hooks/use-pipeline-script.ts       script text ↔ store
  stores/use-script.store.ts         { script, initialScript } — dirty-check only
  ui/PipelineJobsRoute.tsx           composes jobs' JobsPage + the Run action
  ui/dashboard/                      DashboardPipeline.page, PipelineChart,
                                     PipelineDashboardHeader, pipeline-table/
  ui/editor/                         PipelineCreation.page, PipelineUpdate.page,
                                     PipelineEditor, PipelineEditorHeader
  ui/editor/blueprint/               ReactFlow canvas, nodes, edges, node dialogs
  utils/blueprint-converter.ts       graph ↔ PipelineStep[]
  utils/create-default-script.ts
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `project` | index | `LIST_PIPELINES_BY_PROJECT` | `DashboardPipelinePage` |
| `project` | `create` | `CREATE_PIPELINE` | `PipelineCreationPage` |
| `project` | `edit/:pipelineId` | `UPDATE_PIPELINE` | `PipelineUpdatePage` |
| `project` | `pipelines/:pipelineId/jobs` | `LIST_JOBS_BY_PIPELINE` | `PipelineJobsRoute` |

**No nav entry** — pipelines are the project index, reached by entering a project.

The jobs route is owned here, not by [`jobs`](../jobs/AGENTS.md): the page needs a **Run**
action, which is a pipeline operation. `PipelineJobsRoute` renders `JobsPage` (imported from
`jobs`'s public API) and supplies that action.

## Rules that bite here

- **Query keys come from `pipelines.query-keys.ts`** and are exported publicly so other modules
  invalidate what this one reads. After `run()`, also invalidate `JOBS_QUERY_ROOT` /
  `JOBS_QUERY_KEY(pipelineId)` from `features/jobs` — a run creates a job.
- **ReactFlow and CodeMirror are genuine outside-React systems.** `useEffect` is legitimate in
  `use-blueprint-state.ts` and the editor. That licence does not extend to the rest of the
  module.
- **`use-script.store.ts` holds `script` + `initialScript` only** — draft editor text and the
  baseline for the dirty check. It is UI state, not server state. Never put the fetched
  `PipelineEntity` in it.
- `blueprint-converter.ts` is the single translation between the ReactFlow graph and
  `PipelineStep[]`. Both directions live there; do not inline conversion in a component.
- Largest module in the codebase — respect the ~150-line component rule. New editor UI goes in
  `ui/editor/blueprint/`, new list UI in `ui/dashboard/pipeline-table/`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
