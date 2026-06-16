import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreateProject = () => {
  const queryClient = useQueryClient();
  const { createProject } = useDependencies().project;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ name, organizationId, description }: { name: string; organizationId: string; description?: string }) =>
      (await createProject.execute(name, organizationId, description)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PROJECT_CREATE));
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
