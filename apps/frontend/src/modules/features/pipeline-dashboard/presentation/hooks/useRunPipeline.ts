import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { JOBS_QUERY_KEY } from '@/modules/features/pipeline-dashboard/presentation/hooks/usePipelinesJobs.ts';

export const useRunPipeline = () => {
  const { runPipeline } = useDependencies().pipelineDashboard;
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (pipelineId: string) => (await runPipeline.execute(pipelineId)).unwrap(),
    onSuccess: (_data, pipelineId) => {
      toast.success(`Pipeline successfully run`);
      queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId), exact: true });
    },
  });
};
