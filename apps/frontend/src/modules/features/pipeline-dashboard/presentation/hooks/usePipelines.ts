import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';
import type { ListPipelinesResponse } from '@/generated/pipeline.ts';

export const usePipelines = () => {
  const { getPipelines } = useDependencies().pipelineDashboard;

  const {
    data: pipelines,
    isLoading,
    error,
    isError,
  } = useQuery<ListPipelinesResponse, ScyllaError>({
    queryKey: ['pipelines'],
    queryFn: async () => (await getPipelines.execute()).unwrap(),
    staleTime: 0,
    gcTime: 0,
  });

  return {
    pipelines: pipelines?.pipelines,
    isLoading,
    isError,
    error,
    errorMessage: error instanceof Error ? error.message : 'Une erreur est survenue',
  };
};
