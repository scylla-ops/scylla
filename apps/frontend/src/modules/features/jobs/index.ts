/**
 * Pipeline runs: their status, their logs, and the live tail of both.
 *
 * The public API of the module. The hooks are gone (Phase 3): their
 * replacements are options objects with no framework in them, so `dashboard`
 * and `pipeline` — still React — hand one straight to `useQuery` / `useQueries`
 * and share the same cache entry as the Svelte pages.
 *
 * `loadJobsPage` is a **loader**, not the component. `pipeline` composes this
 * page behind its own route (it owns the "Run" action), and a `.svelte`
 * re-exported from a barrel cannot be tree-shaken out of whoever imports that
 * barrel for something else — `use-run-pipeline.ts` imports `JOBS_QUERY_KEY`
 * from here. A function keeps the chunk separate. See `refacto_svelte.md` §2.
 */
export type { JobEntity } from './domain/entities/job.entity.ts';
export type { JobLog, JobLogStream } from './domain/structs/job.struct.ts';
export type { JobsSummary, JobStatus } from './domain/structs/jobs-summary.struct.ts';
export {
  summarizeJobs,
  isActiveStatus,
  isFinishedStatus,
} from './domain/structs/jobs-summary.struct.ts';
export {
  JOBS_QUERY_KEY,
  ORGANIZATION_JOBS_QUERY_KEY,
  JOBS_QUERY_ROOT,
} from './presentation/jobs.query-keys.ts';
export { jobQueries, asJobFeed, ORGANIZATION_JOBS_WINDOW } from './presentation/jobs.queries.ts';
export { jobsByPipelinesQueries } from './presentation/jobs-by-pipelines.queries.ts';

/** `() => import(…)` — the chunk is fetched when a route actually mounts it. */
export const loadJobsPage = () => import('./presentation/ui/Jobs.page.svelte');
