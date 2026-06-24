import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

/** Enable/disable a trigger, with an optimistic toggle in the cached list. */
export const useSetTriggerEnabled = (pipelineId: string) => {
  const { setTriggerEnabled } = useDependencies().triggers;
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ triggerId, enabled }: { triggerId: string; enabled: boolean }) =>
      (await setTriggerEnabled.execute(triggerId, enabled)).unwrap(),
    onMutate: async ({ triggerId, enabled }) => {
      const key = TRIGGERS_QUERY_KEY(pipelineId);
      await queryClient.cancelQueries({ queryKey: key });
      const previous = queryClient.getQueryData<TriggerEntity[]>(key);
      queryClient.setQueryData<TriggerEntity[]>(key, current =>
        (current ?? []).map(trigger =>
          trigger.id === triggerId ? { ...trigger, enabled } : trigger,
        ),
      );
      return { previous };
    },
    onError: (_error, _variables, context) => {
      if (context?.previous) {
        queryClient.setQueryData(TRIGGERS_QUERY_KEY(pipelineId), context.previous);
      }
    },
    onSuccess: (_data, { enabled }) => {
      toast.success(
        i18n._(enabled ? ToastMessages.TRIGGER_ENABLED : ToastMessages.TRIGGER_DISABLED),
      );
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
