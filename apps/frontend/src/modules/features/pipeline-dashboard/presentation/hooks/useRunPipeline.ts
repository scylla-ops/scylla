import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies';
import { useMutation } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useRunPipeline = () => {
  const { runPipeline } = useDependencies().pipelineDashboard;

  return useMutation({
    mutationFn: async (pipelineId: string) => (await runPipeline.execute(pipelineId)).unwrap(),
    onSuccess: () => {
      toast.success(`Pipeline successfully run`);
    },
  });
};
