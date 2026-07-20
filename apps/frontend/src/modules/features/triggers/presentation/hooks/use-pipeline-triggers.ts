import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { TriggerEntity } from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import { TriggerKind } from '@/modules/features/triggers/domain/structs/trigger-source.struct.ts';

export const TRIGGERS_QUERY_KEY = (pipelineId: string) =>
  ['triggers', 'pipeline', pipelineId] as const;

/** List a pipeline's triggers. Polls while an enabled cron trigger exists, to keep `nextFireAt`/`lastResult` fresh. */
export const usePipelineTriggers = (pipelineId: string) => {
  const { listPipelineTriggers } = useDependencies().triggers;

  const { data, isLoading, isError, error, refetch } = useQuery<TriggerEntity[], ScyllaError>({
    queryKey: TRIGGERS_QUERY_KEY(pipelineId),
    enabled: !!pipelineId,
    queryFn: async () => (await listPipelineTriggers.execute(pipelineId)).unwrap(),
    staleTime: 30 * 1000,
    refetchInterval: query => {
      const triggers = query.state.data ?? [];
      const hasEnabledCron = triggers.some(
        trigger => trigger.enabled && trigger.source.kind === TriggerKind.Cron,
      );
      return hasEnabledCron ? 30_000 : false;
    },
  });

  return {
    triggers: data ?? [],
    isLoading,
    isError,
    error,
    refetch,
  };
};
