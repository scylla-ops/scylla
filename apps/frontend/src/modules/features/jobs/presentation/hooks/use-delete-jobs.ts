import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';

export const useDeleteJobs = (pipelineId?: string) => {
  const queryClient = useQueryClient();
  const { deleteJob } = useDependencies().jobs;

  return useMutation({
    mutationFn: async (jobId: string) => (await deleteJob.execute(jobId)).unwrap(),
    onSuccess: () => {
      if (pipelineId) {
        void queryClient.invalidateQueries({ queryKey: ['jobs', 'pipeline', pipelineId] });
      }
    },
  });
};
