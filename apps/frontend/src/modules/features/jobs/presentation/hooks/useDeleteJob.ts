import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useDeleteJob = (pipelineId?: string) => {
  const queryClient = useQueryClient();
  const { deleteJob } = useDependencies().jobs;

  return useMutation({
    mutationFn: async (jobId: string) => (await deleteJob.execute(jobId)).unwrap(),
    onSuccess: () => {
      if (pipelineId) {
        queryClient.invalidateQueries({ queryKey: ['jobs', 'pipeline', pipelineId] });
      }
    },
  });
};

