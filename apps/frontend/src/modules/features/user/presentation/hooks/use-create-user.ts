import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { toast } from '@shared/presentation/utils/toast.ts';

export const useCreateUser = () => {
  const queryClient = useQueryClient();
  const { createUser } = useDependencies().user;

  return useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) =>
      (await createUser.execute(username, password)).unwrap(),
    onSuccess: () => {
      toast.success('User created');
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
    onError: (error: unknown) => {
      let message = 'Failed to create user';

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
          // If decoding fails, use the original message
        }
      } else if (error instanceof Error) {
        message = error.message;
      }
      toast.error(message);
    },
  });
};
