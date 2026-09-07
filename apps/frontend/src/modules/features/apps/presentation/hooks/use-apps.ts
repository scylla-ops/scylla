import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useAppsDomain } from '@/modules/features/apps/presentation/hooks/use-apps-domain.ts';
import { useContextStore } from '@platform/context/use-context.store.ts';

const APPS_QUERY_KEY = 'apps';

export function useApps() {
  const { appsRepository } = useAppsDomain();
  const organizationId = useContextStore(state => state.organization.id);
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: [APPS_QUERY_KEY, organizationId],
    enabled: !!organizationId,
    queryFn: async () => {
      const result = await appsRepository.listApps(organizationId ?? '');
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
      const result = await appsRepository.createApp(organizationId ?? '', name);
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
      const result = await appsRepository.deleteApp(appId);
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
      const result = await appsRepository.setAppActive(appId, active);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    onSuccess: (_data, { appId }) => {
      void queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, organizationId] });
      void queryClient.invalidateQueries({ queryKey: [APPS_QUERY_KEY, 'detail', appId] });
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
  const { appsRepository } = useAppsDomain();

  return useQuery({
    queryKey: [APPS_QUERY_KEY, 'detail', appId],
    enabled: !!appId,
    queryFn: async () => {
      const result = await appsRepository.getApp(appId);
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
  const { appsRepository } = useAppsDomain();
  const queryClient = useQueryClient();
  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, appId] });

  const query = useQuery({
    queryKey: [SECRETS_QUERY_KEY, appId],
    enabled: !!appId,
    queryFn: async () => {
      const result = await appsRepository.listAppSecrets(appId);
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
      const result = await appsRepository.createAppSecret(appId, label);
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
      const result = await appsRepository.revokeAppSecret(secretId);
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
      const result = await appsRepository.setAppSecretEnabled(secretId, enabled);
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
