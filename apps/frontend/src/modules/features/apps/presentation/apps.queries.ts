import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { AppEntity, AppSecretEntity } from '../domain/entities/app.entity.ts';
import type { CreatedApp, CreatedAppSecret } from '../domain/structs/app.struct.ts';
import type { AppsModule } from '../apps.module.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what `use-apps.ts` was — three hooks bundling seven operations —
 * with the framework taken out: `queryOptions` / `mutationOptions` describe the
 * call, `createQuery` / `createMutation` run it inside a component, and a test
 * can exercise a `queryFn` on its own.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () => getModuleDomain<typeof AppsModule.domain>('apps').appsRepository;

export const APPS_QUERY_KEY = (organizationId: string) => ['apps', organizationId] as const;
export const APP_QUERY_KEY = (appId: string) => ['apps', 'detail', appId] as const;
export const APP_SECRETS_QUERY_KEY = (appId: string) => ['app-secrets', appId] as const;

export const appQueries = {
  /** An organization's apps. */
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

  /**
   * One app's secrets — **metadata only**. The plaintext is returned once, at
   * creation, and never again.
   */
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
  /** The plaintext passes through here once, on its way out. Never stored. */
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

  /**
   * Disabling is reversible, deleting is not — they are distinct operations at
   * the repository and must stay distinct in the UI.
   */
  setActive: (organizationId: string) =>
    mutationOptions({
      mutationFn: async ({ appId, active }: { appId: string; active: boolean }) =>
        (await repository().setAppActive(appId, active)).unwrap(),
      onSuccess: (_data, { appId }) => {
        // Two cache entries, one change: the list and the detail are separate
        // queries and both show the flag that just moved.
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

  /** Irreversible, and it cuts any session using the secret. Confirm first. */
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
