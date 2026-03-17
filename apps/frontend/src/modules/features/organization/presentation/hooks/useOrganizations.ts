// modules/organisation/presentation/hooks/useOrganizations.ts
import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useOrganizations = () => {
  const { getOrganizations } = useDependencies().organization;

  const {
    data: organizations,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['organizations'],
    queryFn: async () => (await getOrganizations.execute()).unwrap(),
    staleTime: 1000 * 60 * 5, // 5 minutes TODO: change
  });

  return {
    organizations: organizations?.organizations,
    isLoading,
    isError: !!error,
  };
};
