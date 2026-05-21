import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

const WORKERS_QUERY_KEY = 'workers';

export function useWorkers() {
  const { workers } = useDependencies();
  const organizationId = useContextStore(state => state.organization.id);
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: [WORKERS_QUERY_KEY, organizationId],
    enabled: !!organizationId,
    queryFn: async () => {
      const result = await workers.getWorkers.execute(organizationId ?? '');
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });

  const createWorker = useMutation({
    mutationFn: async (name: string) => {
      const result = await workers.createWorker.execute(organizationId ?? '', name);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: [WORKERS_QUERY_KEY, organizationId] }),
  });

  const deleteWorker = useMutation({
    mutationFn: async (workerId: string) => {
      const result = await workers.deleteWorker.execute(workerId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: [WORKERS_QUERY_KEY, organizationId] }),
  });

  return {
    workers: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createWorker,
    deleteWorker,
  };
}

export function useWorker(workerId: string) {
  const { workers } = useDependencies();

  return useQuery({
    queryKey: [WORKERS_QUERY_KEY, 'detail', workerId],
    enabled: !!workerId,
    queryFn: async () => {
      const result = await workers.getWorker.execute(workerId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });
}

export function useWorkerStats(workerId: string) {
  const { workers } = useDependencies();

  return useQuery({
    queryKey: [WORKERS_QUERY_KEY, 'stats', workerId],
    enabled: !!workerId,
    queryFn: async () => {
      const result = await workers.getWorkerStats.execute(workerId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });
}
