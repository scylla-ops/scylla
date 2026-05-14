import { type Query, useQueries } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMemo } from 'react';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

const MAX_JOBS_PER_PIPELINE = 10;

export const JOBS_QUERY_KEY = (pipelineId: string) =>
  ['jobs', 'pipeline', pipelineId, MAX_JOBS_PER_PIPELINE] as const;

/**
 * Fetches jobs for multiple pipelines in parallel.
 * Returns a map of pipelineId → JobResponse[] for an easy lookup.
 */
export const usePipelineJobs = (pipelineIds: string[]) => {
  const { getPipelineJobs } = useDependencies().jobs;

  const queries = useQueries({
    queries: pipelineIds.map(pipelineId => ({
      queryKey: JOBS_QUERY_KEY(pipelineId),
      queryFn: async () => {
        const result = await getPipelineJobs.execute(pipelineId, {
          page: 1,
          pageSize: MAX_JOBS_PER_PIPELINE,
        });

        const items = result.unwrap().items;
        return { pipelineId, jobs: items };
      },
      staleTime: 0,
      refetchInterval: (query: Query<{ pipelineId: string; jobs: Job[] }, Error>) => {
        const data = query.state.data;

        if (!data) return false;

        const hasActive = data.jobs.some(j => j.status === 'running' || j.status === 'pending');

        return hasActive ? 2000 : false;
      },
    })),
  });

  const isLoading = queries.some(q => q.isLoading);
  const isError = queries.some(q => q.isError);

  const jobsByPipelineId = useMemo(() => {
    const map = new Map<string, Job[]>();
    for (const query of queries) {
      if (query.data) {
        map.set(query.data.pipelineId, query.data.jobs);
      }
    }
    return map;
  }, [queries]);

  //todo: error by pipeline id instead of global
  return { jobsByPipelineId, isJobsLoading: isLoading, isJobsError: isError };
};
