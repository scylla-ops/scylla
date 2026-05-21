import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

const APPS_QUERY_KEY = 'apps';

export function useApps() {
  const { apps } = useDependencies();
  const organizationId = useContextStore(state => state.organization.id);
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: [APPS_QUERY_KEY, organizationId],
    enabled: !!organizationId,
    queryFn: async () => {
      const result = await apps.getApps.execute(organizationId ?? '');
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });

  const createApp = useMutation({
    mutationFn: async (name: string) => {
      const result = await apps.createApp.execute(organizationId ?? '', name);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, organizationId] }),
  });

  const deleteApp = useMutation({
    mutationFn: async (appId: string) => {
      const result = await apps.deleteApp.execute(appId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, organizationId] }),
  });

  return {
    apps: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createApp,
    deleteApp,
  };
}

export function useApp(appId: string) {
  const { apps } = useDependencies();

  return useQuery({
    queryKey: [APPS_QUERY_KEY, 'detail', appId],
    enabled: !!appId,
    queryFn: async () => {
      const result = await apps.getApp.execute(appId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });
}
