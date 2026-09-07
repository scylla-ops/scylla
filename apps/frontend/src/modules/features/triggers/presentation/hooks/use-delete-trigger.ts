import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTriggersDomain } from '@/modules/features/triggers/presentation/hooks/use-triggers-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';

/** Delete a trigger. */
export const useDeleteTrigger = (pipelineId: string) => {
  const { triggersRepository } = useTriggersDomain();
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (triggerId: string) => (await triggersRepository.deleteById(triggerId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.TRIGGER_DELETE));
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
