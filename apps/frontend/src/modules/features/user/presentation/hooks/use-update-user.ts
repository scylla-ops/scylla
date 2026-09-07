import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useUserDomain } from '@/modules/features/user/presentation/hooks/use-user-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  const { userRepository } = useUserDomain();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ userId, username }: { userId: string; username?: string }) => {
      const result = await userRepository.update(userId, username);
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
