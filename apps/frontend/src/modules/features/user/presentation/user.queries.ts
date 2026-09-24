import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { Permission, authorizationReady, can } from '@platform/authz';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { UserModule } from '../user.module.ts';

const repository = () => getModuleDomain<typeof UserModule.domain>('user').userRepository;

export const USERS_QUERY_KEY = () => ['users'] as const;
export const USER_QUERY_KEY = (userId?: string) => ['user', userId] as const;

export const userQueries = {
  /** Checks `LIST_USERS` itself (used outside this module's route guard). An empty list can mean "not allowed": see `canListUsers`. */
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
