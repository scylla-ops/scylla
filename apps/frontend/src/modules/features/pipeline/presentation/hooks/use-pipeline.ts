import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useQuery } from '@tanstack/react-query';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

export const usePipeline = (pipelineId?: string) => {
  const { getPipeline } = useDependencies().pipeline;

  if (!pipelineId) {
    return {
      pipeline: undefined,
      isLoading: false,
      isError: false,
      error: undefined,
    };
  }

  const { data, isLoading, isError, error } = useQuery<Pipeline, ScyllaError>({
    queryKey: ['current-pipeline'],
    queryFn: async () => (await getPipeline.execute(pipelineId)).unwrap(),
    staleTime: 1000 * 30, // 30 seconds
  });

  return {
    pipeline: data,
    isLoading,
    isError,
    error,
  };
};
