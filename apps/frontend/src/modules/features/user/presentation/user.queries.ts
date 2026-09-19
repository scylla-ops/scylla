import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { Permission, authorizationReady, can } from '@platform/authz';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { UserModule } from '../user.module.ts';

/**
 * The user directory's reads and writes, as plain options objects.
 *
 * These are what the five `use-*` hooks were. Being data rather than hooks is
 * what lets the modules that have not been migrated yet keep consuming them:
 * `roles` and `membership` pass `userQueries.list()` straight to react-query's
 * `useQuery`, which is the same options object svelte-query's `createQuery`
 * takes. One declaration, both bindings, one cache entry.
 */
const repository = () => getModuleDomain<typeof UserModule.domain>('user').userRepository;

export const USERS_QUERY_KEY = () => ['users'] as const;
export const USER_QUERY_KEY = (userId?: string) => ['user', userId] as const;

export const userQueries = {
  /**
   * The whole directory.
   *
   * **The query asks for the permission itself**, rather than trusting its
   * caller: it is exported through the barrel and consumed from `roles` and
   * `membership`, whose pages were entered on `MANAGE_ROLES` and
   * `LIST_ORGANIZATION_MEMBERS` — outside this module's route guard. Listing
   * users is a system-wide capability and the backend checks it on every call.
   *
   * An empty result therefore means "none" *or* "not allowed to look"; a caller
   * that must tell them apart calls {@link canListUsers}.
   */
  list: (options: { enabled?: boolean } = {}) =>
    queryOptions({
      queryKey: USERS_QUERY_KEY(),
      queryFn: async () => (await repository().getAll()).unwrap(),
      enabled: (options.enabled ?? true) && canListUsers(),
    }),

  byId: (userId?: string) =>
    queryOptions({
      queryKey: USER_QUERY_KEY(userId),
      queryFn: async () => {
        if (!userId) throw new Error('User ID is required');
        return (await repository().getById(userId)).unwrap();
      },
      enabled: !!userId,
    }),
};

/** Whether the directory may be read at all — see {@link userQueries.list}. */
export const canListUsers = (): boolean => authorizationReady() && can(Permission.LIST_USERS);

export const userMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async ({ username, password }: { username: string; password: string }) =>
        (await repository().create(username, password)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.USER_CREATE));
        return getQueryClient().invalidateQueries({ queryKey: USERS_QUERY_KEY() });
      },
    }),

  /** No password: changing one is not exposed through this repository. */
  update: () =>
    mutationOptions({
      mutationFn: async ({ userId, username }: { userId: string; username?: string }) =>
        (await repository().update(userId, username)).unwrap(),
      onSuccess: (_result, variables) => {
        toast.success(i18n._(ToastMessages.USER_UPDATE));
        return getQueryClient().invalidateQueries({
          queryKey: USER_QUERY_KEY(variables.userId),
        });
      },
    }),

  remove: () =>
    mutationOptions({
      mutationFn: async (userId: string) => (await repository().delete(userId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.USER_DELETE));
        return getQueryClient().invalidateQueries({ queryKey: USERS_QUERY_KEY() });
      },
    }),
};
