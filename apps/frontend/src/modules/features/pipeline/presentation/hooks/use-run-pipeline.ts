import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { JOBS_QUERY_KEY } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-jobs.ts';

export const useRunPipeline = () => {
  const { runPipeline } = useDependencies().pipeline;
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (pipelineId: string) => (await runPipeline.execute(pipelineId)).unwrap(),
    onSuccess: (_data, pipelineId) => {
      toast.success(`Pipeline successfully run`);
      queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId), exact: true });
    },
  });
};
