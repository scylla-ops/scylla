import { useQueries } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { JobResponse } from '@/generated/job.ts';
import { useMemo } from 'react';

const MAX_JOBS_PER_PIPELINE = 10;

export const JOBS_QUERY_KEY = (pipelineId: string) =>
  ['jobs', 'pipeline', pipelineId, MAX_JOBS_PER_PIPELINE] as const;

/**
 * Fetches jobs for multiple pipelines in parallel.
 * Returns a map of pipelineId → JobResponse[] for easy lookup.
 */
export const usePipelinesJobs = (pipelineIds: string[]) => {
  const { getPipelineJobs } = useDependencies().jobs;

  const queries = useQueries({
    queries: pipelineIds.map(pipelineId => ({
      queryKey: [...JOBS_QUERY_KEY(pipelineId)],
      queryFn: async () => {
        const result = await getPipelineJobs.execute(pipelineId, {
          page: 1,
          pageSize: MAX_JOBS_PER_PIPELINE,
        });
        return { pipelineId, jobs: result.unwrap().jobs };
      },
      staleTime: 0,
      refetchInterval: (query: { state: { data?: { pipelineId: string; jobs: JobResponse[] } } }) => {
        const data = query.state.data;
        if (!data) return false;
        const hasActive = data.jobs.some(j => j.status === 'running' || j.status === 'pending');
        return hasActive ? 5000 : false;
      },
    })),
  });

  const isLoading = queries.some(q => q.isLoading);
  const isError = queries.some(q => q.isError);

  const jobsByPipelineId = useMemo(() => {
    const map = new Map<string, JobResponse[]>();
    for (const query of queries) {
      if (query.data) {
        map.set(query.data.pipelineId, query.data.jobs);
      }
    }
    return map;
  }, [queries]);

  return { jobsByPipelineId, isLoading, isError };
};
