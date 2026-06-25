import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useQuery } from '@tanstack/react-query';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

export const usePipeline = (pipelineId: string) => {
  const { getPipeline } = useDependencies().pipeline;

  const { data, isLoading, isError, error } = useQuery<PipelineEntity, ScyllaError>({
    queryKey: ['pipeline', pipelineId],
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
