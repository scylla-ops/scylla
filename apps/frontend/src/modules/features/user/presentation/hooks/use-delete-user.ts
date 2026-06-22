import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  const { deleteUser } = useDependencies().user;
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (userId: string) => (await deleteUser.execute(userId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.USER_DELETE));
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};
