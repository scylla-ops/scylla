import { i18n } from '@lingui/core';
import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { TriggerDraft, TriggerEntity } from '../domain/entities/trigger.entity.ts';
import { TriggerKind } from '../domain/structs/trigger-source.struct.ts';
import type { TriggersModule } from '../triggers.module.ts';

// Resolved per call: tests swap the registry.
const repository = () =>
  getModuleDomain<typeof TriggersModule.domain>('triggers').triggersRepository;

export const TRIGGERS_QUERY_KEY = (pipelineId: string) =>
  ['triggers', 'pipeline', pipelineId] as const;

/** Must match `JOBS_QUERY_KEY` of `jobs`: not imported, to keep the graph acyclic. */
const JOBS_OF_PIPELINE = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;

export const triggerQueries = {
  /** Polls only while an enabled cron trigger exists: nothing else changes by itself. */
  byPipeline: (pipelineId: string) =>
    queryOptions<TriggerEntity[]>({
      queryKey: TRIGGERS_QUERY_KEY(pipelineId),
      enabled: !!pipelineId,
      queryFn: async () => (await repository().listByPipelineId(pipelineId)).unwrap(),
      staleTime: 30 * 1000,
      refetchInterval: query => {
        const triggers = query.state.data ?? [];
        const hasEnabledCron = triggers.some(
          trigger => trigger.enabled && trigger.source.kind === TriggerKind.Cron,
        );
        return hasEnabledCron ? 30_000 : false;
      },
    }),
};

const invalidateTriggers = (pipelineId: string) =>
  getQueryClient().invalidateQueries({ queryKey: TRIGGERS_QUERY_KEY(pipelineId) });

export const triggerMutations = {
  create: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async (draft: TriggerDraft) =>
        (await repository().create(pipelineId, draft)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_CREATE));
        void invalidateTriggers(pipelineId);
      },
    }),

  update: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async ({ triggerId, draft }: { triggerId: string; draft: TriggerDraft }) =>
        (await repository().update(triggerId, draft)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_UPDATE));
        void invalidateTriggers(pipelineId);
      },
    }),

  /** A deleted webhook URL is gone forever, unlike disabling. Confirm first. */
  remove: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async (triggerId: string) =>
        (await repository().deleteById(triggerId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_DELETE));
        void invalidateTriggers(pipelineId);
      },
    }),

  /** Optimistic: the switch moves at once, and `onError` rolls it back. */
  setEnabled: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async ({ triggerId, enabled }: { triggerId: string; enabled: boolean }) =>
        (await repository().setEnabled(triggerId, enabled)).unwrap(),
      onMutate: async ({ triggerId, enabled }) => {
        const queryClient = getQueryClient();
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
          getQueryClient().setQueryData(TRIGGERS_QUERY_KEY(pipelineId), context.previous);
        }
      },
      onSuccess: (_data, { enabled }) => {
        toast.success(
          i18n._(enabled ? ToastMessages.TRIGGER_ENABLED : ToastMessages.TRIGGER_DISABLED),
        );
      },
      onSettled: () => void invalidateTriggers(pipelineId),
    }),

  /** Creates a real job: the pipeline's job list is invalidated too. */
  fireNow: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async (triggerId: string) => (await repository().fireNow(triggerId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_FIRED));
        void getQueryClient().invalidateQueries({ queryKey: JOBS_OF_PIPELINE(pipelineId) });
        void invalidateTriggers(pipelineId);
      },
    }),
};
