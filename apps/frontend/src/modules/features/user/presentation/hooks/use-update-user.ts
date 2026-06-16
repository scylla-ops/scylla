import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  const { updateUser } = useDependencies().user;

  return useMutation({
    mutationFn: async ({ userId, username }: { userId: string; username?: string }) => {
      const result = await updateUser.execute(userId, username);
      return result.unwrap();
    },
    onSuccess: (_, variables) => {
      toast.success('User information updated');
      void queryClient.invalidateQueries({
        queryKey: ['user', variables.userId],
      });
    },
  });
};
