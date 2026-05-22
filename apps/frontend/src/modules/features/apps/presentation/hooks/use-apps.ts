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

  const setAppActive = useMutation({
    mutationFn: async ({ appId, active }: { appId: string; active: boolean }) => {
      const result = await apps.setAppActive.execute(appId, active);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: (_data, { appId }) => {
      queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, organizationId] });
      queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, 'detail', appId] });
    },
  });

  return {
    apps: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createApp,
    deleteApp,
    setAppActive,
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

const SECRETS_QUERY_KEY = 'app-secrets';

/** Secrets of one app: list query + create/revoke/enable mutations. */
export function useAppSecrets(appId: string) {
  const { apps } = useDependencies();
  const queryClient = useQueryClient();
  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, appId] });

  const query = useQuery({
    queryKey: [SECRETS_QUERY_KEY, appId],
    enabled: !!appId,
    queryFn: async () => {
      const result = await apps.listAppSecrets.execute(appId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });

  const createSecret = useMutation({
    mutationFn: async (label: string) => {
      const result = await apps.createAppSecret.execute(appId, label);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: invalidate,
  });

  const revokeSecret = useMutation({
    mutationFn: async (secretId: string) => {
      const result = await apps.revokeAppSecret.execute(secretId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: invalidate,
  });

  const setSecretEnabled = useMutation({
    mutationFn: async ({ secretId, enabled }: { secretId: string; enabled: boolean }) => {
      const result = await apps.setAppSecretEnabled.execute(secretId, enabled);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: invalidate,
  });

  return {
    secrets: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createSecret,
    revokeSecret,
    setSecretEnabled,
  };
}
