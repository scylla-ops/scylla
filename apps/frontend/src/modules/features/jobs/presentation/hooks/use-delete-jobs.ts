import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useJobsDomain } from '@/modules/features/jobs/presentation/hooks/use-jobs-domain.ts';

export const useDeleteJobs = (pipelineId?: string) => {
  const queryClient = useQueryClient();
  const { jobsRepository } = useJobsDomain();

  return useMutation({
    mutationFn: async (jobId: string) => (await jobsRepository.deleteById(jobId)).unwrap(),
    onSuccess: () => {
      if (pipelineId) {
        void queryClient.invalidateQueries({ queryKey: ['jobs', 'pipeline', pipelineId] });
      }
    },
  });
};
