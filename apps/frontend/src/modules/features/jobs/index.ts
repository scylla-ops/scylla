/**
 * Pipeline runs: their status, their logs, and the live tail of both.
 *
 * The public API of the module. `pipeline` composes `JobsPage` behind its own
 * route (it owns the "Run" action), which is why a page is exported here — the
 * only such case, and its single consumer is itself lazily loaded.
 */
export type { JobEntity } from './domain/entities/job.entity.ts';
export type { JobLog, JobLogStream } from './domain/structs/job.struct.ts';
export type { JobsSummary, JobStatus } from './domain/structs/jobs-summary.struct.ts';
export { summarizeJobs, isActiveStatus, isFinishedStatus } from './domain/structs/jobs-summary.struct.ts';
export {
  JOBS_QUERY_KEY,
  ORGANIZATION_JOBS_QUERY_KEY,
  JOBS_QUERY_ROOT,
} from './presentation/hooks/jobs.query-keys.ts';
export { useOrganizationJobs } from './presentation/hooks/use-organization-jobs.ts';
export { useJobsByPipelines } from './presentation/hooks/use-jobs-by-pipelines.ts';
export { JobsPage } from './presentation/ui/Jobs.page.tsx';
