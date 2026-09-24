# `features/jobs` — agent guide

Pipeline runs: their status, their logs, and the live tail of both.

**Layer** `features/` · **id** `jobs` · **DI key** `jobsRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type JobEntity
type JobLog, JobLogStream
type JobsSummary, JobStatus
summarizeJobs, isActiveStatus, isFinishedStatus
JOBS_QUERY_KEY, ORGANIZATION_JOBS_QUERY_KEY, JOBS_QUERY_ROOT
jobQueries, asJobFeed, ORGANIZATION_JOBS_WINDOW
jobsByPipelinesQueries
loadJobsPage                          ← a loader, not the page
```

**This module is Svelte** (Phase 3). There is no `use-jobs-domain.ts` and no hooks: the
factories in `presentation/jobs.queries.ts` resolve the repository through
`getModuleDomain('jobs')` per call, and a component runs them with `createQuery` /
`createMutation` from `@platform/query`.

`dashboard` and `pipeline` run the same factories with `createQuery` / `createQueries`, and
share the *same* cache entry as this module's pages.

`loadJobsPage` is a **loader** (`() => import(…)`), never the component. `pipeline` composes this
page behind its own route because it owns the "Run" action, and a `.svelte` re-exported from a
barrel cannot be tree-shaken out of whoever imports that barrel for something else —
`use-run-pipeline.ts` imports `JOBS_QUERY_KEY` from here. A function keeps the chunk separate.
See `refacto_svelte.md` §2. `JobDetails.page.svelte` is mounted by this module's own route, so it
stays private.

Never add: `jobs.module.ts`, and never export a `.svelte` component from this barrel.

## Data contract

`JobsRepository` — `domain/repository/jobs.repository.ts`:

| Method | Returns |
|---|---|
| `getByPipelineId(...)` | paginated `JobEntity` list |
| `getByOrganizationId(...)` | paginated `JobEntity` list |
| `getById(jobId)` | `JobEntity` |
| `deleteById(jobId)` | `void` |
| `getLogs(...)` | `JobLog[]` |
| `tailLogs(jobId, nodeId?)` | **`ScyllaResult<JobLogStream>` — synchronous, not a Promise** |

`tailLogs` is the one non-`Promise` method in the codebase: it opens a server stream and returns
the handle immediately. It is a genuine outside-the-framework subscription, so it belongs in
`tail-job-logs.svelte.ts` — and **the stream is always closed on teardown**.

All wrapped in `ScyllaResult`. Reach it from `jobs.queries.ts` only — never from a component.

## Layout

```
jobs.module.ts                       DI wiring + the job details route (no nav)
index.ts                             public API
domain/
  entities/job.entity.ts             JobEntity
  structs/job.struct.ts              JobLog, JobLogStream
  structs/jobs-summary.struct.ts     JobsSummary, JobStatus + summarizeJobs,
                                     isActiveStatus, isFinishedStatus (pure)
  repository/jobs.repository.ts
infrastructure/
  repository/data-sources/jobs-remote.data-source.ts    interface
  data/remote/grpc-jobs-remote.data-source.ts           impl
  repository/mappers/grpc-job.mapper.ts
  repository/default-jobs.repository.ts
presentation/
  jobs.query-keys.ts                 key factories — import these, never inline a key
  jobs.queries.ts                    every read and write, as query/mutation options
  jobs-by-pipelines.queries.ts       the fan-out across pipelines, avoids N+1
  open-log-panels.svelte.ts          which node panels are open, mirrored in the URL
  tail-job-logs.svelte.ts            the live stream subscription (opens and closes it)
  ui/Jobs.page.svelte, JobsHeader.svelte
  ui/JobDetails.page.svelte          one job: info + per-node logs
  ui/jobs.messages.ts                every string — `lingui extract` cannot read `.svelte`
  ui/job-details/                    JobSummary, JobNodeLogs
  ui/jobs-table/                     JobsTable + job-columns.ts + cells
  ui/jobs-table/job-timeline.calculator.ts   segment geometry, pure and tested in node
  ui/jobs-log/JobLogDisplay.svelte   the streamed viewer, shared by both pages
  ui/jobs-log/streamed-log-view.svelte.ts    owns the editor document + tail-follow
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `project` | `pipelines/:pipelineId/jobs/:jobId` | `READ_JOB` | `JobDetailsPage` |

No nav entry. The jobs **list** is not declared here: it is mounted by
[`pipeline`](../pipeline/AGENTS.md) at `project` → `pipelines/:pipelineId/jobs`, gated on
`LIST_JOBS_BY_PIPELINE`, via `PipelineJobsRoute`, because that page needs a Run action. If you
change `JobsPage`'s props, that is the call site to update.

One job needs nothing from `pipeline`, so its route lives here — a flat sibling of the list route
rather than a child of it, since two modules cannot share one path segment when the parent
already renders a page of its own.

## Rules that bite here

- **Query keys come from `jobs.query-keys.ts`.** Four exist; three (`JOBS_QUERY_KEY`,
  `ORGANIZATION_JOBS_QUERY_KEY`, `JOBS_QUERY_ROOT`) are exported publicly so other modules
  invalidate the *same* key this module queries, and `JOB_QUERY_KEY` (one job, read by the
  details page) stays internal. Never hand-write `['jobs', ...]`.
- `JOBS_QUERY_ROOT` is the prefix for "invalidate everything jobs-related" after a run;
  `JOBS_QUERY_KEY(pipelineId)` is exact. Pick deliberately.
- **`summarizeJobs`, `isActiveStatus`, `isFinishedStatus` are pure domain functions.** Status
  logic goes there, not into a component or a `useMemo`.
- **Log tailing is a subscription to something outside the framework**, so it is a Svelte
  action / `*.svelte.ts` owner — `tail-job-logs.svelte.ts` — and never reactivity. It opens the
  stream and **closes it on teardown**; the same holds for `streamed-log-view.svelte.ts`, which
  drives a CodeMirror instance through `@shared/presentation/ui/editor/code-mirror.actions.ts`.
- **Never re-seed the editor with the whole live log string.** Replacing the document
  (`changes: { from: 0, to: doc.length }`) resets the scroll offset and collapses the selection;
  at one flush per 150 ms the log appears to jump back to the top and nothing can be selected.
  `streamed-log-view.svelte.ts` appends the delta instead — that is the entire reason it exists.
  This is also why the CodeMirror mount is an action: the editor is an outside system, and a
  `value` prop would invite exactly the replacement above.
- **Never scroll that viewer to the bottom on every log change** either — same flush rate, and it
  steals the viewport from a user who scrolled up or is dragging a selection. The same file owns
  that decision.
- **The log viewer is sized by its caller, never by itself.** `JobLogDisplay` takes a `maxHeight`
  in pixels and grows with the log up to it; the details page gives the whole job's panel the
  column it measured, and every node panel one fixed readable height whatever the count — the
  column scrolls rather than shrink them. A fixed height on the editor itself is what this
  replaced — do not put one back.
- Lists go through `DataTable` + pagination. Row keys are job ids, never indices.
- **The details page is the one place a job's logs are read.** Everywhere a job is displayed —
  the jobs table's view action and timeline, the pipeline dashboard's history and last run —
  links there through `scyllaNavigate.goToJobDetails(...)` from `@platform/context`, which is an
  object, not a hook. Which log panels are open is the `nodes` search param, not component
  state (`open-log-panels.svelte.ts` owns it), so those links can open straight onto one node's
  logs.
- **The whole job and the nodes are exclusive.** The details page opens a `JobLogDisplay` per node
  the URL names, and the job as a whole only when it names none — the whole job is the page at
  rest, never a panel standing alongside the nodes, which is why its panel has no close button.
  Closing the last node panel is what comes back to it.
- **The timeline picks one node, the log nav builds a set.** A click on a timeline segment shows
  that node alone (`selectNode`) — the same jump the links from the jobs list and the pipeline
  dashboard make, landing on the page rather than moving inside it. The nav under the summary
  adds and removes panels instead (`toggleNode`), which is what lets two nodes' output be read
  side by side.
- **Log panels are independent, and each one is a live stream.** Every open panel holds its own
  tail subscription. Closing a panel unmounts it, which is what cancels that stream — never hide
  one with CSS instead.
- **New strings go in `ui/jobs.messages.ts`, never inside a `.svelte`.** Extraction does not read
  components, so a message declared there vanishes from the catalogs without failing a gate.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
