import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { AppEntity, AppSecretEntity } from '../domain/entities/app.entity.ts';
import type { CreatedApp, CreatedAppSecret } from '../domain/structs/app.struct.ts';
import type { AppsModule } from '../apps.module.ts';

// Resolved per call: tests swap the registry.
const repository = () => getModuleDomain<typeof AppsModule.domain>('apps').appsRepository;

export const APPS_QUERY_KEY = (organizationId: string) => ['apps', organizationId] as const;
export const APP_QUERY_KEY = (appId: string) => ['apps', 'detail', appId] as const;
export const APP_SECRETS_QUERY_KEY = (appId: string) => ['app-secrets', appId] as const;

export const appQueries = {
  byOrganization: (organizationId: string) =>
    queryOptions<AppEntity[]>({
      queryKey: APPS_QUERY_KEY(organizationId),
      enabled: !!organizationId,
      queryFn: async () => (await repository().listApps(organizationId)).unwrap(),
    }),

  byId: (appId: string) =>
    queryOptions<AppEntity>({
      queryKey: APP_QUERY_KEY(appId),
      enabled: !!appId,
      queryFn: async () => (await repository().getApp(appId)).unwrap(),
    }),

  /** Metadata only. */
  secretsOf: (appId: string) =>
    queryOptions<AppSecretEntity[]>({
      queryKey: APP_SECRETS_QUERY_KEY(appId),
      enabled: !!appId,
      queryFn: async () => (await repository().listAppSecrets(appId)).unwrap(),
    }),
};

const invalidateList = (organizationId: string) =>
  getQueryClient().invalidateQueries({ queryKey: APPS_QUERY_KEY(organizationId) });

const invalidateSecrets = (appId: string) =>
  getQueryClient().invalidateQueries({ queryKey: APP_SECRETS_QUERY_KEY(appId) });

export const appMutations = {
  /** The value passes through once and is never stored. */
  create: (organizationId: string) =>
    mutationOptions({
      mutationFn: async (name: string): Promise<CreatedApp> =>
        (await repository().createApp(organizationId, name)).unwrap(),
      onSuccess: () => void invalidateList(organizationId),
    }),

  remove: (organizationId: string) =>
    mutationOptions({
      mutationFn: async (appId: string) => (await repository().deleteApp(appId)).unwrap(),
      onSuccess: () => void invalidateList(organizationId),
    }),

  /** Reversible, unlike deleting. */
  setActive: (organizationId: string) =>
    mutationOptions({
      mutationFn: async ({ appId, active }: { appId: string; active: boolean }) =>
        (await repository().setAppActive(appId, active)).unwrap(),
      onSuccess: (_data, { appId }) => {
        // The list and the detail both show the flag.
        void invalidateList(organizationId);
        void getQueryClient().invalidateQueries({ queryKey: APP_QUERY_KEY(appId) });
      },
    }),

  createSecret: (appId: string) =>
    mutationOptions({
      mutationFn: async (label: string): Promise<CreatedAppSecret> =>
        (await repository().createAppSecret(appId, label)).unwrap(),
      onSuccess: () => void invalidateSecrets(appId),
    }),

  /** Irreversible, and cuts any session using the secret. Confirm first. */
  revokeSecret: (appId: string) =>
    mutationOptions({
      mutationFn: async (secretId: string) =>
        (await repository().revokeAppSecret(secretId)).unwrap(),
      onSuccess: () => void invalidateSecrets(appId),
    }),

  setSecretEnabled: (appId: string) =>
    mutationOptions({
      mutationFn: async ({ secretId, enabled }: { secretId: string; enabled: boolean }) =>
        (await repository().setAppSecretEnabled(secretId, enabled)).unwrap(),
      onSuccess: () => void invalidateSecrets(appId),
    }),
};
