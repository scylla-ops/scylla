// modules/organisation/presentation/hooks/useOrganizations.ts
import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';

export const useOrganizations = () => {
  // Member-scoped: orgs the current user belongs to. Non-admins are denied the
  // global listOrganizations, so the switcher must use listUserOrganizations.
  const { getUserOrganizations } = useDependencies().organization;

  const {
    data: organizations,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['organizations', 'mine'],
    queryFn: async () => (await getUserOrganizations.execute()).unwrap(),
    staleTime: 1000 * 60 * 5, // 5 minutes TODO: change
  });

  return {
    organizations: organizations?.organizations,
    isLoading,
    isError: !!error,
  };
};
