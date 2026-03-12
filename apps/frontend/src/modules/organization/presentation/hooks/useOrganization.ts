// modules/organisation/presentation/hooks/useOrganization.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

export const useOrganization = () => {
  const queryClient = useQueryClient();

  // 1. Récupération (Query)
  const {
    data: organisations,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['organisations'],
    queryFn: () => {},
    staleTime: 1000 * 60 * 5, // 5 minutes de cache "frais"
  });

  // 2. Création (Mutation)
  const createMutation = useMutation({
    mutationFn: () => {},
    onSuccess: () => {
      // Invalidation du cache pour rafraîchir la liste automatiquement
      queryClient.invalidateQueries({ queryKey: ['organisations'] });
    },
  });

  return {
    organisations: organisations ?? [],
    isLoading,
    isError: !!error,
    createOrganisation: createMutation.mutate,
    isCreating: createMutation.isPending,
  };
};
