import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useUpdateOrganization = () => {
  const queryClient = useQueryClient();
  const { updateOrganization } = useDependencies().organization;

  return useMutation({
    mutationFn: async ({ organizationId, name, description }: { organizationId: string; name?: string; description?: string }) =>
      (await updateOrganization.execute(organizationId, name, description)).unwrap(),
    onSuccess: () => {
      toast.success('Organization updated');
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
    onError: () => toast.error('Failed to update organization'),
  });
};

