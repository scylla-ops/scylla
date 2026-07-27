import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { PrincipalKind } from '@/modules/features/permission/domain/structs/permission.struct.ts';

export const MY_PERMISSIONS_QUERY_KEY = (userId: string) =>
  ['permission-effective', userId] as const;

/**
 * The current user's effective permissions across every scope. Loaded once and
 * cached — it backs the app-wide {@link useAuthorization} checks (nav visibility,
 * button gating, …).
 */
export const useMyPermissions = () => {
  const { permission } = useDependencies();
  const userId = localStorage.getItem('userId') ?? '';

  const query = useQuery({
    queryKey: MY_PERMISSIONS_QUERY_KEY(userId),
    queryFn: async () =>
      (
        await permission.getEffectivePermissions.execute({ kind: PrincipalKind.USER, id: userId })
      ).unwrap(),
    enabled: userId !== '',
    staleTime: 1000 * 60 * 5,
  });

  return {
    // No user id (query disabled) → settled with no permissions, not "loading".
    effective: userId === '' ? { scopes: [] } : query.data,
    isLoading: query.isLoading,
    isError: query.isError,
  };
};
