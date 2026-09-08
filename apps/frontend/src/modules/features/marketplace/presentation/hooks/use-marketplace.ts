import { useMarketplaceDomain } from '@/modules/features/marketplace/presentation/hooks/use-marketplace-domain.ts';
import { useQuery } from '@tanstack/react-query';

export const useMarketplace = () => {
  const { marketplaceRepository } = useMarketplaceDomain();

  return useQuery({
    queryKey: ['marketplace'],
    queryFn: async () => {
      const result = await marketplaceRepository.getMarketplace();
      return result.unwrap();
    },
    staleTime: 1000 * 60,
    retry: 1,
  });
};
