import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useOrganizationDomain } from '@/modules/features/organization/presentation/hooks/use-organization-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@platform/context';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreateOrganization = () => {
  const queryClient = useQueryClient();
  const { organizationRepository } = useOrganizationDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ name, description }: { name: string; description?: string }) =>
      (await organizationRepository.create(name, description)).unwrap(),
    onSuccess: data => {
      useContextStore.getState().setOrganization(data.id, data.name);
      toast.success(i18n._(ToastMessages.ORGANIZATION_CREATE));
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
