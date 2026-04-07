import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import type { ScyllaError } from '@/modules/shared/utils/ScyllaResult.ts';

export const useCreateProject = () => {
  const queryClient = useQueryClient();
  const { createProject } = useDependencies().project;

  return useMutation({
    mutationFn: async ({ name, organizationId }: { name: string; organizationId: string }) =>
      (await createProject.execute(name, organizationId)).unwrap(),
    onSuccess: () => {
      // Opt: return org and update cache
      /* queryClient.setQueryData(['organizations'], (old: any) => {
        return old ? { ...old, organizations: [...old.organizations, newOrg] } : old;
      });*/
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
    onError: (err: ScyllaError) => err.log(),
  });
};
