import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';
import { useQuery } from '@tanstack/react-query';

export const useUsers = () => {
  const { getUsers } = useDependencies().user;

  const {
    data: users,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['users'],
    queryFn: async () => {
      return (await getUsers.execute()).unwrap();
    },
  });

  return {
    users,
    isLoading,
    isError: !!error,
    error,
  };
};
