import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreateProject = () => {
  const queryClient = useQueryClient();
  const { projectRepository } = useProjectDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ name, organizationId, description }: { name: string; organizationId: string; description?: string }) =>
      (await projectRepository.create(name, organizationId, description)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PROJECT_CREATE));
      return queryClient.invalidateQueries({ queryKey: ['projects'] });
    },
  });
};
