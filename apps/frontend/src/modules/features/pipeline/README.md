# Pipeline

> [Scylla frontend](../../../../README.md) › `features/` › **pipeline** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A pipeline is the definition of what Scylla runs: an ordered graph of steps belonging to a
project. This is the largest module in the frontend, and it covers both halves of the
lifecycle — authoring a pipeline and watching it run.

## What it covers

- **The project's pipeline dashboard** (the project index route) — every pipeline in the
  project, with its last run, its status and a chart.
- **The editor** — creating and updating a pipeline, in two coordinated views.
- **Running** — the Run action, and the jobs page that follows it.
- **Organization-wide reads** — pipeline metadata across every project, for the
  [dashboard](../dashboard/README.md).

## Metadata vs entity

The most important modelling decision here is that a pipeline has two shapes:

- **`PipelineMetadata`** — id, name, project, status. What a *list* needs.
- **`PipelineEntity`** — the whole thing, including its `PipelineStep[]`. What the *editor*
  needs.

The repository exposes them through different methods, and lists never fetch the full entity.
On a project with fifty pipelines that is the difference between fifty step graphs crossing the
wire and none. If you are adding a column to a list, add it to the metadata; reach for
`getById` only when you genuinely need the steps.

## The editor: two views of one definition

`PipelineEditor` presents a pipeline as both a **visual graph** and a **script**, and the two
stay in sync.

The graph is [ReactFlow](https://reactflow.dev): `BlueprintCanvas` with custom nodes
(`StartNode`, `PipelineStepNode`), custom edges (`DeletableEdge`), and dialogs for editing a
node's configuration. `use-blueprint-state.ts` owns the nodes and edges, and
`blueprint-converter.ts` is the single place that translates between a ReactFlow graph and the
domain's `PipelineStep[]` — both directions, so the round-trip can't drift.

The script side is CodeMirror, themed from [shared](../../shared/README.md)'s
`use-code-mirror-theme`. `use-script.store.ts` is a small Zustand store holding `script` and
`initialScript`: the draft text and the baseline it is compared against, which is how the editor
knows whether there are unsaved changes. That is UI state, and it is the right use of a store —
the *saved* pipeline stays in TanStack Query.

ReactFlow and CodeMirror are also the codebase's clearest legitimate use of `useEffect`. Both
are systems outside React that have to be synchronized with imperatively. That licence stops at
the editor's edge: the rest of the module derives during render like everywhere else.

## Owning the jobs route

`/:org/projects/:projectId/pipelines/:pipelineId/jobs` is declared by **this** module, not by
[jobs](../jobs/README.md), even though it shows a jobs table.

The reason is the Run button. Running is a pipeline operation. If `jobs` owned the route it
would need a pipeline mutation, which means importing `pipeline` for a concern that isn't its
own. Instead `PipelineJobsRoute` wraps `JobsPage` — imported from `jobs`'s public API — and
supplies the Run action itself. `JobsPage` is one of only two pages in the codebase exported
from a feature barrel, and this is its single consumer.

## Cache coordination

`PIPELINES_QUERY_KEY`, `ORGANIZATION_PIPELINES_QUERY_KEY` and `PIPELINES_QUERY_ROOT` are part of
the public API so other modules invalidate exactly what this one reads.

Running a pipeline is the interesting case: it creates a *job*, so `useRunPipeline` invalidates
the jobs keys as well as its own. That cross-module invalidation only works because both modules
export their key factories instead of writing arrays inline.

## Routes

| Path | Permission | What |
|---|---|---|
| project index | `LIST_PIPELINES_BY_PROJECT` | the pipeline dashboard |
| `create` | `CREATE_PIPELINE` | new pipeline |
| `edit/:pipelineId` | `UPDATE_PIPELINE` | editor |
| `pipelines/:pipelineId/jobs` | `LIST_JOBS_BY_PIPELINE` | runs, with Run |

No sidebar entry: pipelines *are* the project, reached by entering one.

## Related modules

- [jobs](../jobs/README.md) — what a run produces.
- [triggers](../triggers/README.md) — what starts a run without a human.
- [secret](../secret/README.md) — values injected into a run.
- [project](../project/README.md) — the owning scope.
- [marketplace](../marketplace/README.md) — templates to start from.
