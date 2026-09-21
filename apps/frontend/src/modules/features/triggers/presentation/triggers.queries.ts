import { i18n } from '@lingui/core';
import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { TriggerDraft, TriggerEntity } from '../domain/entities/trigger.entity.ts';
import { TriggerKind } from '../domain/structs/trigger-source.struct.ts';
import type { TriggersModule } from '../triggers.module.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what the six hooks under `presentation/hooks/` were, with the
 * framework taken out: `queryOptions` / `mutationOptions` describe the call,
 * `createQuery` / `createMutation` run it inside a component, and a test can
 * exercise a `queryFn` on its own.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () =>
  getModuleDomain<typeof TriggersModule.domain>('triggers').triggersRepository;

export const TRIGGERS_QUERY_KEY = (pipelineId: string) =>
  ['triggers', 'pipeline', pipelineId] as const;

/**
 * The jobs key, spelled out rather than imported from `features/jobs`.
 *
 * Firing a trigger mints a real job, so its list has to be refreshed — but
 * importing the key would make `triggers` depend on `jobs` for one array, and
 * the graph is cheaper kept acyclic. `jobs`' own `JOBS_QUERY_KEY` is the
 * authority; this must match it.
 */
const JOBS_OF_PIPELINE = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;

export const triggerQueries = {
  /**
   * A pipeline's triggers.
   *
   * Polls **only while an enabled cron trigger exists**: `nextFireAt` and
   * `lastResult` move on their own then, and nothing else here does. A webhook
   * trigger changes when someone calls it, which no interval can predict
   * usefully.
   */
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
  /** Answers the `CreatedTrigger` so a caller can reveal a one-time webhook secret. */
  create: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async (draft: TriggerDraft) =>
        (await repository().create(pipelineId, draft)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_CREATE));
        void invalidateTriggers(pipelineId);
      },
    }),

  /** Updates the editable fields: name, source spec, inputs. */
  update: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async ({ triggerId, draft }: { triggerId: string; draft: TriggerDraft }) =>
        (await repository().update(triggerId, draft)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_UPDATE));
        void invalidateTriggers(pipelineId);
      },
    }),

  /**
   * Deleting a webhook trigger invalidates its URL forever — distinct from
   * {@link triggerMutations.setEnabled}, which is reversible. Confirm first.
   */
  remove: (pipelineId: string) =>
    mutationOptions({
      mutationFn: async (triggerId: string) =>
        (await repository().deleteById(triggerId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.TRIGGER_DELETE));
        void invalidateTriggers(pipelineId);
      },
    }),

  /**
   * Enable/disable, with an optimistic toggle in the cached list.
   *
   * The switch has to move under the pointer: a round trip's worth of "nothing
   * happened" on a toggle reads as a broken control, and the rollback in
   * `onError` is what makes the optimism honest.
   */
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

  /**
   * Fires a trigger immediately. **This mints a real job**, so the pipeline's
   * job list is invalidated too — otherwise the run the user just started does
   * not appear anywhere.
   */
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
