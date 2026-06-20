import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeletePipeline = () => {
  const queryClient = useQueryClient();
  const { deletePipeline } = useDependencies().pipeline;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (pipelineId: string) => (await deletePipeline.execute(pipelineId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PIPELINE_DELETE));
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] });
    },
  });
};
