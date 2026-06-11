import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useDeletePipeline = () => {
  const queryClient = useQueryClient();
  const { deletePipeline } = useDependencies().pipeline;

  return useMutation({
    mutationFn: async (pipelineId: string) => (await deletePipeline.execute(pipelineId)).unwrap(),
    onSuccess: () => {
      toast.success('Pipeline deleted');
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] });
    },
  });
};
