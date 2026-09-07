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
useOrganizationJobs, useJobsByPipelines
JobsPage                              ← the documented page exception
```

`JobsPage` is exported because `pipeline` composes it behind its own route (it owns the Run
action). That consumer is itself lazily loaded, which is what makes the exception safe. Do not
add a second page export.

Never add: `jobs.module.ts`, `use-jobs-domain.ts`.

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
the handle immediately. Consume it in an effect (a genuine outside-React subscription) and
**always close it in the cleanup**.

Reach the repository with `useJobsDomain()` **inside a hook only**.

## Layout

```
jobs.module.ts                       DI wiring only — no routes, no nav
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
  hooks/jobs.query-keys.ts           key factories — import these, never inline a key
  hooks/use-jobs-domain.ts           DI accessor (private)
  hooks/use-job.ts, use-organization-jobs.ts, use-pipelines-jobs.ts,
  hooks/use-jobs-by-pipelines.ts     useQueries fan-out, avoids N+1
  hooks/use-job-logs.ts, use-tail-job-logs.ts, use-delete-jobs.ts
  stores/use-jobs.store.ts           UI state only
  ui/Jobs.page.tsx, JobsHeader.tsx
  ui/jobs-table/                     JobsTable + columns + cells (index.ts is local, not public)
  ui/jobs-table/jobs-log/            JobLogDialog, JobLogDisplay
```

## Routes & nav

**None.** `JobsModule` declares `domain` only. The page is mounted by
[`pipeline`](../pipeline/AGENTS.md) at `project` → `pipelines/:pipelineId/jobs`, gated on
`LIST_JOBS_BY_PIPELINE`, via `PipelineJobsRoute`. If you change `JobsPage`'s props, that is the
call site to update.

## Rules that bite here

- **Query keys come from `jobs.query-keys.ts`.** Three exist (`JOBS_QUERY_KEY`,
  `ORGANIZATION_JOBS_QUERY_KEY`, `JOBS_QUERY_ROOT`) and they are exported publicly so other
  modules invalidate the *same* key this module queries. Never hand-write `['jobs', ...]`.
- `JOBS_QUERY_ROOT` is the prefix for "invalidate everything jobs-related" after a run;
  `JOBS_QUERY_KEY(pipelineId)` is exact. Pick deliberately.
- **`summarizeJobs`, `isActiveStatus`, `isFinishedStatus` are pure domain functions.** Status
  logic goes there, not into a component or a `useMemo`.
- Log tailing is a subscription: `useEffect` is *correct* here. It is one of the few places in
  the codebase where it is.
- Lists go through `DataTable` + `usePagination()`. Row keys are job ids, never indices.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
