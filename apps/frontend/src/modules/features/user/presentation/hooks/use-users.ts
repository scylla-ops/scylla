import { useUserDomain } from '@/modules/features/user/presentation/hooks/use-user-domain.ts';
import { useQuery } from '@tanstack/react-query';
import { Permission, useAuthorization } from '@platform/authz';

/**
 * The user directory. Listing it is a system-wide capability — the backend
 * checks `ListUsers` on every call — so the query asks for itself rather than
 * trusting its caller.
 *
 * That check is not redundant with the route guard on `/users`: this hook is
 * exported through the barrel and consumed from `roles` and `membership`, whose
 * pages were entered on `MANAGE_ROLES` and `LIST_ORGANIZATION_MEMBERS`. Two of
 * those call sites used to ask anyway and take the denial.
 *
 * Like `useAgents`, an empty `users` therefore means "none" *or* "not allowed to
 * look" — a caller that needs to tell them apart reads `canListUsers`.
 */
export const useUsers = (options: { enabled?: boolean } = {}) => {
  const { userRepository } = useUserDomain();
  const { can, ready } = useAuthorization();

  const canListUsers = ready && can(Permission.LIST_USERS);

  const {
    data: users,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      return (await userRepository.getAll()).unwrap();
    },
    enabled: (options.enabled ?? true) && canListUsers,
  });

  return {
    users,
    isLoading,
    isError: !!error,
    error,
    canListUsers,
  };
};
