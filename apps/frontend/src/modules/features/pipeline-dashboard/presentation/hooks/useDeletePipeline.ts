import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useDeletePipeline = () => {
  const queryClient = useQueryClient();
  const { deletePipeline } = useDependencies().pipelineDashboard;

  return useMutation({
    mutationFn: async (pipelineId: string) => (await deletePipeline.execute(pipelineId)).unwrap(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pipelines'] });
    },
  });
};
