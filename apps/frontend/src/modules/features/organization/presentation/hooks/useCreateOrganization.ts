import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@core/utils/ScyllaResult.ts';

export const useCreateOrganization = () => {
  const queryClient = useQueryClient();
  const { createOrganization } = useDependencies().organization;

  return useMutation({
    mutationFn: async (name: string) => (await createOrganization.execute(name)).unwrap(),
    onSuccess: () => {
      // Opt: return org and update cache
      /* queryClient.setQueryData(['organizations'], (old: any) => {
        return old ? { ...old, organizations: [...old.organizations, newOrg] } : old;
      });*/
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
    onError: (err: ScyllaError) => err.log(),
  });
};
