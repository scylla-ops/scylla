import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies';
import { useMutation } from '@tanstack/react-query';

export const useRunPipeline = () => {
  const { runPipeline } = useDependencies().pipelineDashboard;

  return useMutation({
    mutationFn: async (pipelineId: string) => (await runPipeline.execute(pipelineId)).unwrap(),
    onSuccess: () => {
      console.log('Pipeline run successfully');
      //todo: toast
    },
  });
};
