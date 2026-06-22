import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';

/** Delete a trigger. */
export const useDeleteTrigger = (pipelineId: string) => {
  const { deleteTrigger } = useDependencies().triggers;
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (triggerId: string) => (await deleteTrigger.execute(triggerId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.TRIGGER_DELETE));
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
