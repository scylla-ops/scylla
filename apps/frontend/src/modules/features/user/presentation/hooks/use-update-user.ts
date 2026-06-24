import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  const { updateUser } = useDependencies().user;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ userId, username }: { userId: string; username?: string }) => {
      const result = await updateUser.execute(userId, username);
      return result.unwrap();
    },
    onSuccess: (_, variables) => {
      toast.success(i18n._(ToastMessages.USER_UPDATE));
      void queryClient.invalidateQueries({
        queryKey: ['user', variables.userId],
      });
    },
  });
};
