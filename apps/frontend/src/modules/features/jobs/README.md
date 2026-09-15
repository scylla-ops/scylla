# Jobs

> [Scylla frontend](../../../../README.md) › `features/` › **jobs** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A **job** is one execution of a pipeline. This module covers everything about runs: listing
them, reading their status, following their logs while they happen, and deleting them.

## What it covers

- **The jobs table** — runs for a pipeline or across an organization, paginated, with status,
  timing and the nodes each run touched.
- **Status semantics** — what counts as running, what counts as finished, and how a list of
  runs collapses into a single summary.
- **Logs** — the full log of a finished run, and the **live tail** of one still in flight.
- **Cleanup** — deleting selected runs.

## Where it appears

`JobsModule` declares no routes. The page is mounted by
[pipeline](../pipeline/README.md) at `/:org/projects/:projectId/pipelines/:pipelineId/jobs`,
behind `PipelineJobsRoute`.

That inversion is deliberate. The jobs page needs a **Run** button, and running is a pipeline
operation, not a job one. Rather than have `jobs` import `pipeline` to get a mutation — or
duplicate the mutation — `pipeline` owns the route, brings its own Run action, and composes
`JobsPage` inside it. `JobsPage` is therefore one of only two pages in the codebase exported
from a feature barrel, and it is safe because its single consumer is itself lazily loaded.

## How it is built

**Domain** holds `JobEntity`, the log types (`JobLog`, `JobLogStream`) and — more interestingly
— `jobs-summary.struct.ts`, which is pure logic with no I/O: `summarizeJobs` turns a list of
runs into counts, `isActiveStatus` and `isFinishedStatus` classify a status. Because these are
plain functions in the domain, the same rules apply on the jobs table, in the pipeline dashboard
and on the org overview, and there is no second opinion about what "running" means.

**Infrastructure** uses the fuller layout — a `data-sources/` interface, a `data/remote/` gRPC
implementation, mappers under `repository/mappers/` — because streaming makes the transport
non-trivial. `tailLogs` is the odd one out in the whole codebase: it returns
`ScyllaResult<JobLogStream>` **synchronously**, since it opens a server stream rather than
awaiting a response. The presentation layer subscribes to it in an effect and closes it on
cleanup, which is exactly the kind of outside-React system effects are for.

**Presentation** is the largest layer: a query-key factory module, one hook per operation, a
Zustand store for table UI state, and the `jobs-table/` folder holding the table, its columns
and its cells (`JobStatus`, `JobTimeline`, `JobNodesList`, `JobActions`).

The log viewer behaves like an IDE console, and `useStreamedLogView` is where that lives. A live
log is a stream, but a React `value` prop is a snapshot, and the gap between the two is the whole
problem: handing the growing string to `<ReactCodeMirror value>` makes it replace the entire
document on every flush — seven times a second — which resets the scroll offset and destroys any
selection the reader had made. So the hook appends the delta itself and keeps the prop frozen.

On top of that it keeps one piece of state — are we still following the tail? — because "always
scroll to the bottom" and "let me read this line" are in direct conflict. Any gesture that moves
away from the end turns following off, any return to the end turns it back on, and a selection
being made holds it off regardless.

`useJobsByPipelines` is worth knowing about: it fetches runs for many pipelines with
`useQueries` rather than a loop of `useQuery`, which is how the pipeline dashboard shows a "last
run" column without an N+1 storm.

## Query keys are part of the contract

`JOBS_QUERY_KEY`, `ORGANIZATION_JOBS_QUERY_KEY` and `JOBS_QUERY_ROOT` are exported from the
module's public API, not kept private. That is on purpose: when `pipeline` triggers a run, it
must invalidate *the same* cache entries this module reads. Sharing the factory is what keeps
"I clicked Run and the table updated" true. Hand-writing a key array anywhere else silently
breaks it.

## Related modules

- [pipeline](../pipeline/README.md) — owns the route, the Run action, and what a job executes.
- [agents](../agents/README.md) — the machines that pick jobs up.
- [dashboard](../dashboard/README.md) — consumes `useOrganizationJobs` for the overview.

## Related shared pieces

`DataTable`, `usePagination`, `useSelection` and `FeatureHeader` — see
[shared](../../shared/README.md).
