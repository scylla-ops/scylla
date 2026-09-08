import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useUserDomain } from '@/modules/features/user/presentation/hooks/use-user-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDeleteUser = () => {
  const queryClient = useQueryClient();
  const { userRepository } = useUserDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (userId: string) => (await userRepository.delete(userId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.USER_DELETE));
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};
