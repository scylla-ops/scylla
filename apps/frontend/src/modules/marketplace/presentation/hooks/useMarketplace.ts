import { useDependencies } from '@/modules/core/presentation/hooks/useDependencies.ts';
import { useQuery } from '@tanstack/react-query';

export const useMarketplace = () => {
  const deps = useDependencies();

  return useQuery({
    queryKey: ['marketplace'],
    queryFn: async () => {
      const result = await deps.marketplace.getMarketplaceUseCase.execute();

      if (!result.ok) {
        throw new Error(result.error.message);
      }

      return result.value;
    },
    staleTime: 1000 * 60,
    retry: 1,
  });
};
