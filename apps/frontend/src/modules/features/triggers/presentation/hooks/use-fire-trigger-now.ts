import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';

/**
 * Fire a trigger immediately. Mints a real Job, so we invalidate the pipeline's
 * jobs queries (prefix match covers both the dashboard and the Jobs page) and
 * the triggers list (to refresh `lastFiredAt`/`lastStatus`).
 */
export const useFireTriggerNow = (pipelineId: string) => {
  const { fireTriggerNow } = useDependencies().triggers;
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (triggerId: string) => (await fireTriggerNow.execute(triggerId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.TRIGGER_FIRED));
      void queryClient.invalidateQueries({ queryKey: ['jobs', 'pipeline', pipelineId] });
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
