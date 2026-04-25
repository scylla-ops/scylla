import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  const { deleteUser } = useDependencies().user;

  return useMutation({
    mutationFn: async (userId: string) => (await deleteUser.execute(userId)).unwrap(),
    onSuccess: () => {
      toast.success('User deleted');
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};

