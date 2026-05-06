import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
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
      toast.success('User information updated successfully');
      queryClient.invalidateQueries({
        queryKey: ['user', variables.userId],
      });
    },
    onError: (error: unknown) => {
      let message = 'Failed to update user information';

      if (
        error instanceof ScyllaError &&
        error.cause &&
        typeof error.cause === 'object' &&
        'message' in error.cause
      ) {
        message = error.cause.message as string;

        try {
          message = decodeURIComponent(message);
        } catch {
          // Ignore decode errors
        }
      }

      toast.error(message);
    },
  });
};
