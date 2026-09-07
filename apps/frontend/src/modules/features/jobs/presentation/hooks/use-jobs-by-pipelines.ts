import { type Query, useQueries } from '@tanstack/react-query';
import { useMemo } from 'react';
import { useJobsDomain } from '@/modules/features/jobs/presentation/hooks/use-jobs-domain.ts';
import { JOBS_QUERY_KEY } from '@/modules/features/jobs/presentation/hooks/jobs.query-keys.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { Permission, useAuthorization } from '@platform/authz';

/** Enough recent jobs to draw a pipeline's history strip, not its whole log. */
const MAX_JOBS_PER_PIPELINE = 10;

/**
 * The most recent jobs of several pipelines at once, keyed by pipeline id.
 *
 * Part of the module's public API: the pipeline dashboard draws a job history
 * per row, which is a jobs query. It lives here — with the entity and the query
 * key it reads — so that `pipeline` depends on `jobs` and never the reverse.
 *
 * `ListJobsByPipeline` is enforced per project, so without the grant this
 * fan-out is one guaranteed `PERMISSION_DENIED` per pipeline — an error toast
 * each, and a row of "failed to load" where the honest answer is "you may not
 * see this". Nothing is asked in that case; callers read `canListJobs` to say so.
 */
export const useJobsByPipelines = (pipelineIds: string[]) => {
  const { jobsRepository } = useJobsDomain();
  const { can, ready } = useAuthorization();

  // The page is already scoped to one project, so the ambient target is it.
  const canListJobs = ready && can(Permission.LIST_JOBS_BY_PIPELINE);

  const queries = useQueries({
    queries: pipelineIds.map(pipelineId => ({
      queryKey: JOBS_QUERY_KEY(pipelineId),
      queryFn: async () => {
        const result = await jobsRepository.getByPipelineId(pipelineId, {
          page: 1,
          pageSize: MAX_JOBS_PER_PIPELINE,
        });

        const items = result.unwrap().items;
        return { pipelineId, jobs: items };
      },
      enabled: canListJobs,
      staleTime: 0,
      refetchInterval: (query: Query<{ pipelineId: string; jobs: JobEntity[] }, Error>) => {
        const data = query.state.data;

        if (!data) return false;

        const hasActive = data.jobs.some(j => j.status === 'running' || j.status === 'pending');

        return hasActive ? 2000 : false;
      },
    })),
  });

  // Permissions still unknown → keep the skeletons up, rather than flash an
  // empty history or a denial the user may not actually be under.
  const isLoading = !ready || queries.some(q => q.isLoading);
  const isError = queries.some(q => q.isError);

  const jobsByPipelineId = useMemo(() => {
    const map = new Map<string, JobEntity[]>();
    for (const query of queries) {
      if (query.data) {
        map.set(query.data.pipelineId, query.data.jobs);
      }
    }
    return map;
  }, [queries]);

  //todo: error by pipeline id instead of global
  return { jobsByPipelineId, isJobsLoading: isLoading, isJobsError: isError, canListJobs };
};
