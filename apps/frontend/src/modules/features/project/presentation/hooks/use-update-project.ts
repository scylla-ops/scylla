import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useUpdateProject = () => {
  const queryClient = useQueryClient();
  const { updateProject } = useDependencies().project;

  return useMutation({
    mutationFn: async ({ projectId, name, description }: { projectId: string; name?: string; description?: string }) =>
      (await updateProject.execute(projectId, name, description)).unwrap(),
    onSuccess: () => {
      toast.success('Project updated');
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
    onError: () => toast.error('Failed to update project'),
  });
};

