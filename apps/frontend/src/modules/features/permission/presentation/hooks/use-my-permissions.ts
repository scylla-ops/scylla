import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { PrincipalKind } from '@/modules/features/permission/domain/structs/permission.struct.ts';

export const MY_PERMISSIONS_QUERY_KEY = (userId: string) => ['authz-effective', userId] as const;

/**
 * The current user's effective permissions across every scope. Loaded once and
 * cached — it backs the app-wide {@link useAuthorization} checks (nav visibility,
 * button gating, …).
 */
export const useMyPermissions = () => {
  const { authz } = useDependencies();
  const userId = localStorage.getItem('userId') ?? '';

  const query = useQuery({
    queryKey: MY_PERMISSIONS_QUERY_KEY(userId),
    queryFn: async () =>
      (
        await authz.getEffectivePermissions.execute({ kind: PrincipalKind.USER, id: userId })
      ).unwrap(),
    enabled: userId !== '',
    staleTime: 1000 * 60 * 5,
  });

  return {
    effective: query.data,
    isLoading: query.isLoading,
    isError: query.isError,
  };
};
