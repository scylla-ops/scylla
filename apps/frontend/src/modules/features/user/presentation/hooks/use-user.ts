import { useQuery } from '@tanstack/react-query';
import { useUserDomain } from '@/modules/features/user/presentation/hooks/use-user-domain.ts';

export const useUser = (userId?: string) => {
  const { userRepository } = useUserDomain();

  const {
    data: user,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['user', userId],
    queryFn: async () => {
      if (!userId) {
        throw new Error('User ID is required');
      }
      return (await userRepository.getById(userId)).unwrap();
    },
    enabled: !!userId,
  });

  return {
    user,
    isLoading,
    isError: !!error,
    error,
  };
};
