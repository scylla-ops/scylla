import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useDeleteOrganization = () => {
  const queryClient = useQueryClient();
  const { deleteOrganization } = useDependencies().organization;

  return useMutation({
    mutationFn: async (organizationId: string) =>
      (await deleteOrganization.execute(organizationId)).unwrap(),
    onSuccess: () => {
      toast.success('Organization deleted');
      return queryClient.invalidateQueries({ queryKey: ['organizations'] });
    },
    onError: () => toast.error('Failed to delete organization'),
  });
};

