// modules/organisation/presentation/hooks/useOrganizations.ts
import { useQuery } from '@tanstack/react-query';
import { useOrganizationDomain } from '@/modules/features/organization/presentation/hooks/use-organization-domain.ts';

export const useOrganizations = () => {
  // Member-scoped: orgs the current user belongs to. Non-admins are denied the
  // global listOrganizations, so the switcher must use listUserOrganizations.
  const { organizationRepository } = useOrganizationDomain();

  const {
    data: organizations,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['organizations', 'mine'],
    queryFn: async () => (await organizationRepository.getMine()).unwrap(),
    staleTime: 1000 * 60 * 5, // 5 minutes TODO: change
  });

  return {
    organizations,
    isLoading,
    isError: !!error,
  };
};
