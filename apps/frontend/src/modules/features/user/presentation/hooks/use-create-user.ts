import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
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
  });
};
