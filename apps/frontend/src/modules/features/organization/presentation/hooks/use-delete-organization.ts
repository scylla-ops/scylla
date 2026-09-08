import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useOrganizationDomain } from '@/modules/features/organization/presentation/hooks/use-organization-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeleteOrganization = () => {
  const queryClient = useQueryClient();
  const { organizationRepository } = useOrganizationDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (organizationId: string) =>
      (await organizationRepository.delete(organizationId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.ORGANIZATION_DELETE));
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
