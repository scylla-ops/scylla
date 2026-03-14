// modules/organisation/presentation/hooks/useOrganization.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@core/utils/ScyllaResult.ts';

export const useOrganization = () => {
  const queryClient = useQueryClient();
  const { getOrganizations, createOrganization } = useDependencies().organization;

  const {
    data: organisations,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['organisations'],
    queryFn: async () => (await getOrganizations.execute()).unwrap(),
    staleTime: 1000 * 60 * 5, // 5 minutes TODO: change
  });

  const createMutation = useMutation({
    mutationFn: async (name: string) => (await createOrganization.execute(name)).unwrap(),
    onSuccess: () => {
      //TODO: instead of that, createOrganization usecase return the organization created and we put in tanstack cache the new organization
      return queryClient.invalidateQueries({ queryKey: ['organisations'] });
    },
    onError: (err: ScyllaError) => err.log(),
  });

  return {
    organisations: organisations,
    isLoading,
    isError: !!error,
    createOrganisation: createMutation.mutate,
    isCreating: createMutation.isPending,
  };
};
