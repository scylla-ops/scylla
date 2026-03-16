// modules/organisation/presentation/hooks/useOrganizations.ts
import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/useDependencies.ts';

export const useProjects = () => {
  const { getProjects } = useDependencies().project;

  //TODO: currently refetch every time: change to cache

  const {
    data: projects,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['projects'],
    queryFn: async () => (await getProjects.execute()).unwrap(),
    staleTime: 0,
    gcTime: 0,
    refetchOnMount: true,
  });

  return {
    projects: projects?.projects,
    isLoading,
    isError: !!error,
  };
};
