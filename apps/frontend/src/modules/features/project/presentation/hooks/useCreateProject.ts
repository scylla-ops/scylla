import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useCreateProject = () => {
  const queryClient = useQueryClient();
  const { createProject } = useDependencies().project;

  return useMutation({
    mutationFn: async ({ name, organizationId }: { name: string; organizationId: string }) =>
      (await createProject.execute(name, organizationId)).unwrap(),
    onSuccess: () => {
      toast.success('Project created');
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
