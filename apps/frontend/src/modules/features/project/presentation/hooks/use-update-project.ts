import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useUpdateProject = () => {
  const queryClient = useQueryClient();
  const { updateProject } = useDependencies().project;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({
      projectId,
      name,
      description,
    }: {
      projectId: string;
      name?: string;
      description?: string;
    }) => (await updateProject.execute(projectId, name, description)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PROJECT_UPDATE));
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
