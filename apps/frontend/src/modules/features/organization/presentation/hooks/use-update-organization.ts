import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useOrganizationDomain } from '@/modules/features/organization/presentation/hooks/use-organization-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useUpdateOrganization = () => {
  const queryClient = useQueryClient();
  const { organizationRepository } = useOrganizationDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({
      organizationId,
      name,
      description,
    }: {
      organizationId: string;
      name?: string;
      description?: string;
    }) => (await organizationRepository.update(organizationId, name, description)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.ORGANIZATION_UPDATE));
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
