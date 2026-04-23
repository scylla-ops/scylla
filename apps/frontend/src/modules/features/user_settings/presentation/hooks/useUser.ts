import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useUser = (userId?: string) => {
  const { getUser } = useDependencies().userSettings;

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
      return (await getUser.execute(userId)).unwrap();
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
