import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';
import type { TriggerDraft } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

/** Create a trigger. Returns the CreatedTrigger so callers can reveal a one-time webhook secret. */
export const useCreateTrigger = (pipelineId: string) => {
  const { createTrigger } = useDependencies().triggers;
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (draft: TriggerDraft) =>
      (await createTrigger.execute(pipelineId, draft)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.TRIGGER_CREATE));
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
