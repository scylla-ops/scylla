import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

/**
 * A job's status, as the mapper flattens the wire `state` oneof.
 * `unknown` means the backend sent a state arm newer than this build.
 */
export type JobStatus =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'orphaned'
  | 'unknown';

/** Still going: worth polling for, and excluded from any success rate. */
export const isActiveStatus = (status: string): boolean =>
  status === 'pending' || status === 'running';

/** Reached a terminal state, whatever that state is. */
export const isFinishedStatus = (status: string): boolean =>
  status === 'completed' ||
  status === 'failed' ||
  status === 'cancelled' ||
  status === 'orphaned';

/**
 * Outcome counts over a set of jobs.
 *
 * Deliberately a *window* summary, not an all-time one: the backend exposes no
 * organization-level aggregate (unlike `GetAgentStats` for a single agent), so
 * these are derived from the page of jobs that was actually fetched. Callers
 * that show them must say so — see `isPartialWindow` on `useOrganizationJobs`.
 */
export interface JobsSummary {
  total: number;
  pending: number;
  running: number;
  completed: number;
  failed: number;
  cancelled: number;
  orphaned: number;
  /** Jobs in a terminal state — the denominator of {@link successRate}. */
  finished: number;
  /**
   * Share of finished jobs that completed, in `[0, 1]`.
   * `null` when nothing has finished yet, which is not the same as 0 %.
   */
  successRate: number | null;
  /** Most recent `createdAt` in the set, or `null` when the set is empty. */
  lastRunAt: string | null;
}

const EMPTY: JobsSummary = {
  total: 0,
  pending: 0,
  running: 0,
  completed: 0,
  failed: 0,
  cancelled: 0,
  orphaned: 0,
  finished: 0,
  successRate: null,
  lastRunAt: null,
};

/**
 * Folds a set of jobs into its outcome mix.
 *
 * Pure — no framework, no transport — so it belongs to the domain and is
 * testable on its own. Cancelled and orphaned runs count as failures for the
 * success rate: neither produced the result that was asked for.
 */
export const summarizeJobs = (jobs: readonly JobEntity[]): JobsSummary => {
  if (jobs.length === 0) return EMPTY;

  const summary = { ...EMPTY, total: jobs.length };
  let lastRunAt: string | null = null;

  for (const job of jobs) {
    switch (job.status) {
      case 'pending':
        summary.pending++;
        break;
      case 'running':
        summary.running++;
        break;
      case 'completed':
        summary.completed++;
        break;
      case 'failed':
        summary.failed++;
        break;
      case 'cancelled':
        summary.cancelled++;
        break;
      case 'orphaned':
        summary.orphaned++;
        break;
      // `unknown` is counted in `total` only: guessing a bucket for a state this
      // build does not know would quietly skew the rate.
    }

    if (isFinishedStatus(job.status)) summary.finished++;
    if (lastRunAt === null || job.createdAt > lastRunAt) lastRunAt = job.createdAt;
  }

  return {
    ...summary,
    successRate: summary.finished === 0 ? null : summary.completed / summary.finished,
    lastRunAt,
  };
};
