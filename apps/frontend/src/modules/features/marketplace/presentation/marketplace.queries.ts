import { queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { MarketplaceModule } from '../marketplace.module.ts';

const repository = () =>
  getModuleDomain<typeof MarketplaceModule.domain>('marketplace').marketplaceRepository;

export const MARKETPLACE_QUERY_KEY = () => ['marketplace'] as const;

export const marketplaceQueries = {
  list: () =>
    queryOptions({
      queryKey: MARKETPLACE_QUERY_KEY(),
      queryFn: async () => (await repository().getMarketplace()).unwrap(),
      staleTime: 1000 * 60,
      retry: 1,
    }),
};
