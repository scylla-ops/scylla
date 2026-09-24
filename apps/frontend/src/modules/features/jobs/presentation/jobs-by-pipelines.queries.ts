import { Permission, authorizationReady, can } from '@platform/authz';
import type { JobEntity } from '../domain/entities/job.entity.ts';
import { jobQueries } from './jobs.queries.ts';

interface HistoryResult {
  data?: { pipelineId: string; jobs: JobEntity[] };
  isLoading: boolean;
  isError: boolean;
}

/**
 * The latest jobs of several pipelines. Without `LIST_JOBS_BY_PIPELINE` nothing is
 * asked (one denial per pipeline otherwise): callers read `canListJobs`.
 *
 * In its own file: `feature-permissions.test.ts` looks for `can(` per file, and
 * `jobQueries` deliberately does not check.
 */
export const jobsByPipelinesQueries = (pipelineIds: string[]) => {
  // The page is scoped to one project: the ambient target.
  const canListJobs = authorizationReady() && can(Permission.LIST_JOBS_BY_PIPELINE);

  return {
    canListJobs,
    queries: pipelineIds.map(pipelineId => jobQueries.historyOf(pipelineId, canListJobs)),
    // Folded here so TanStack Query memoizes it on the results.
    combine: (results: HistoryResult[]) => ({
      jobsByPipelineId: new Map(
        results.flatMap(result =>
          result.data ? [[result.data.pipelineId, result.data.jobs] as const] : [],
        ),
      ),
      // Permissions unknown: keep the skeletons.
      isJobsLoading: !authorizationReady() || results.some(result => result.isLoading),
      isJobsError: results.some(result => result.isError),
    }),
  };
};
