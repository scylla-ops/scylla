import { Permission, authorizationReady, can } from '@platform/authz';
import type { JobEntity } from '../domain/entities/job.entity.ts';
import { jobQueries } from './jobs.queries.ts';

interface HistoryResult {
  data?: { pipelineId: string; jobs: JobEntity[] };
  isLoading: boolean;
  isError: boolean;
}

/**
 * The most recent jobs of several pipelines at once, as a fan-out both bindings
 * can run — `pipeline`'s dashboard draws a history strip per row.
 *
 * `ListJobsByPipeline` is enforced per project, so without the grant this is
 * one guaranteed `PERMISSION_DENIED` per pipeline — an error toast each, and a
 * row of "failed to load" where the honest answer is "you may not see this".
 * Nothing is asked in that case, and callers read `canListJobs` to say so.
 *
 * Returns what `useQueries` / `createQueries` take rather than calling either,
 * the way `projectLookupQueries` does — that is what lets `pipeline` keep
 * consuming it from React until Phase 5.
 *
 * **In its own file, and that is load-bearing.** `feature-permissions.test.ts`
 * asks "does this factory check for itself?" by looking for a `can(` in the
 * file that declares it. Sitting next to `jobQueries` — which deliberately does
 * *not* check, being organization-scoped and filtered server-side — this one's
 * gate would have vouched for both, and the ratchet would have gone quiet on
 * exactly the read it exists to watch.
 */
export const jobsByPipelinesQueries = (pipelineIds: string[]) => {
  // The page is already scoped to one project, so the ambient target is it.
  const canListJobs = authorizationReady() && can(Permission.LIST_JOBS_BY_PIPELINE);

  return {
    canListJobs,
    queries: pipelineIds.map(pipelineId => jobQueries.historyOf(pipelineId, canListJobs)),
    // Folded here rather than by the caller so TanStack Query can memoize the
    // map on the underlying results instead of rebuilding it every render.
    combine: (results: HistoryResult[]) => ({
      jobsByPipelineId: new Map(
        results.flatMap(result =>
          result.data ? [[result.data.pipelineId, result.data.jobs] as const] : [],
        ),
      ),
      // Permissions still unknown → keep the skeletons up, rather than flash an
      // empty history or a denial the user may not actually be under.
      isJobsLoading: !authorizationReady() || results.some(result => result.isLoading),
      isJobsError: results.some(result => result.isError),
    }),
  };
};
