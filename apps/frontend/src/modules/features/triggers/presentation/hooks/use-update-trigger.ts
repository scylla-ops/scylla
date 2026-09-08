import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTriggersDomain } from '@/modules/features/triggers/presentation/hooks/use-triggers-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { TRIGGERS_QUERY_KEY } from '@/modules/features/triggers/presentation/hooks/use-pipeline-triggers.ts';
import type { TriggerDraft } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';

/** Update a trigger's editable fields (name, source spec, inputs). */
export const useUpdateTrigger = (pipelineId: string) => {
  const { triggersRepository } = useTriggersDomain();
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ triggerId, draft }: { triggerId: string; draft: TriggerDraft }) =>
      (await triggersRepository.update(triggerId, draft)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.TRIGGER_UPDATE));
      void queryClient.invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });
    },
  });
};
