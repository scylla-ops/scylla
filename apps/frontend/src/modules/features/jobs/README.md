# Jobs

> [Scylla frontend](../../../../README.md) › `features/` › **jobs** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A **job** is one execution of a pipeline. This module covers everything about runs: listing
them, reading their status, following their logs while they happen, and deleting them.

## What it covers

- **The jobs table** — runs for a pipeline or across an organization, paginated, with status,
  timing and the nodes each run touched.
- **The job details page** — one run: its status, its timing, the nodes it touched and their
  logs.
- **Status semantics** — what counts as running, what counts as finished, and how a list of
  runs collapses into a single summary.
- **Logs** — the full log of a finished run, and the **live tail** of one still in flight.
- **Cleanup** — deleting selected runs.

## Where it appears

`JobsModule` declares one route: the job details page, at
`/:org/projects/:projectId/pipelines/:pipelineId/jobs/:jobId`.

A job used to have no page of its own — its status lived in a table row and its logs in a modal
opened from that row, so there was nothing to link to. The details page is now the single place
a run is read, and every surface that shows a job links to it: the view action on the jobs
table, a node in its timeline (which opens straight onto that node's logs, via a `nodes` search
param), and the history and last-run cells on the pipeline dashboard. That param is a list: the
page opens one log panel per node it names, so two nodes' output can be read side by side.

The jobs **list**, though, is mounted by [pipeline](../pipeline/README.md) at
`/:org/projects/:projectId/pipelines/:pipelineId/jobs`, behind `PipelineJobsRoute`.

That inversion is deliberate. The jobs list page needs a **Run** button, and running is a pipeline
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
awaiting a response. The presentation layer subscribes to it in `tail-job-logs.svelte.ts` and
closes it on teardown — a stream is a system outside the framework, and that is what actions and
`*.svelte.ts` owners are for, not reactivity.

**Presentation** is the largest layer: a query-key factory module, the `*.queries.ts` factories
that replaced the hooks, the `jobs-table/` folder holding the table, its columns and its cells
(`JobStatus`, `JobTimeline`, `JobActions`), and `job-details/` holding the two halves of the
details page — `JobSummary` (what the run did) and `JobNodeLogs` (what it printed, per node).
The streamed viewer itself, `jobs-log/JobLogDisplay`, is shared by both pages.

The log viewer behaves like an IDE console, and `streamed-log-view.svelte.ts` is where that
lives. A live log is a stream, but a document you re-seed is a snapshot, and the gap between the
two is the whole problem: replacing the editor's content on every flush — seven times a second —
resets the scroll offset and destroys any selection the reader had made. So it appends the delta
itself and never re-seeds. Mounting the editor is a Svelte action for the same reason: CodeMirror
is an outside system, and a `value` prop would invite exactly that replacement.

On top of that it keeps one piece of state — are we still following the tail? — because "always
scroll to the bottom" and "let me read this line" are in direct conflict. Any gesture that moves
away from the end turns following off, any return to the end turns it back on, and a selection
being made holds it off regardless.

`jobsByPipelinesQueries` is worth knowing about: it describes the runs of many pipelines as a
*list* of query options, so the caller fans them out in one batch rather than looping one call
per pipeline — which is how the pipeline dashboard shows a "last run" column without an N+1
storm. `pipeline` and the pages here run them with `createQueries`, sharing one cache.

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
