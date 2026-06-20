import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreateUser = () => {
  const queryClient = useQueryClient();
  const { createUser } = useDependencies().user;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ username, password }: { username: string; password: string }) =>
      (await createUser.execute(username, password)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.USER_CREATE));
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};
