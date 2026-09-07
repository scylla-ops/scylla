import { useMutation, useQueryClient } from '@tanstack/react-query';
import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeletePipeline = () => {
  const queryClient = useQueryClient();
  const { pipelineRepository } = usePipelineDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (pipelineId: string) => (await pipelineRepository.deleteById(pipelineId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PIPELINE_DELETE));
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] });
    },
  });
};
