import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useQuery } from '@tanstack/react-query';

export const useMarketplace = () => {
  const deps = useDependencies();

  return useQuery({
    queryKey: ['marketplace'],
    queryFn: async () => {
      const result = await deps.marketplace.getMarketplaceUseCase.execute();
      return result.unwrap();
    },
    staleTime: 1000 * 60,
    retry: 1,
  });
};
