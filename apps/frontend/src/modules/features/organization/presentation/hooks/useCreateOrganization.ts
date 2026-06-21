import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreateOrganization = () => {
  const queryClient = useQueryClient();
  const { createOrganization } = useDependencies().organization;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ name, description }: { name: string; description?: string }) =>
      (await createOrganization.execute(name, description)).unwrap(),
    onSuccess: data => {
      useContextStore.getState().setOrganization(idValue(data.organizationId), data.name);
      toast.success(i18n._(ToastMessages.ORGANIZATION_CREATE));
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
