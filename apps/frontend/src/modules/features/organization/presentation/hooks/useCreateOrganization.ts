import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { idValue } from '@core/infrastructure/grpc/wrappers.ts';

export const useCreateOrganization = () => {
  const queryClient = useQueryClient();
  const { createOrganization } = useDependencies().organization;

  return useMutation({
    mutationFn: async ({ name, description }: { name: string; description?: string }) =>
      (await createOrganization.execute(name, description)).unwrap(),
    onSuccess: data => {
      useContextStore.getState().setOrganization(idValue(data.organizationId), data.name);
      toast.success('Organization created');
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
  });
};
