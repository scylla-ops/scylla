import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeleteOrganization = () => {
  const queryClient = useQueryClient();
  const { deleteOrganization } = useDependencies().organization;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (organizationId: string) =>
      (await deleteOrganization.execute(organizationId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.ORGANIZATION_DELETE));
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
