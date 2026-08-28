import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useQuery } from '@tanstack/react-query';

/**
 * The user directory. Listing it is a system-wide capability (`LIST_USERS`), so
 * a caller that only administers one organization or project passes
 * `enabled: false` rather than asking for a denial.
 */
export const useUsers = (options: { enabled?: boolean } = {}) => {
  const { getUsers } = useDependencies().user;

  const {
    data: users,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      return (await getUsers.execute()).unwrap();
    },
    enabled: options.enabled ?? true,
  });

  return {
    users,
    isLoading,
    isError: !!error,
    error,
  };
};
