import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useDeleteProject = () => {
  const queryClient = useQueryClient();
  const { deleteProject } = useDependencies().project;

  return useMutation({
    mutationFn: async (projectId: string) => (await deleteProject.execute(projectId)).unwrap(),
    onSuccess: () => {
      toast.success('Project deleted');
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
