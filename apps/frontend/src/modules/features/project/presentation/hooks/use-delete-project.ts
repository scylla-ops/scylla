import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeleteProject = () => {
  const queryClient = useQueryClient();
  const { deleteProject } = useDependencies().project;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (projectId: string) => (await deleteProject.execute(projectId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PROJECT_DELETE));
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
