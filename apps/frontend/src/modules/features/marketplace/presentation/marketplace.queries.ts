import { queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import type { MarketplaceModule } from '../marketplace.module.ts';

/**
 * The marketplace's reads, declared once and executed by whoever needs them.
 *
 * This is what `use-marketplace.ts` was, minus the framework: `queryOptions`
 * describes the query, `createQuery` runs it, and nothing here is a hook — so
 * the factory is callable from a view model, from a test, or from
 * `queryClient.fetchQuery` without a component in sight.
 *
 * The repository is resolved **inside** `queryFn`, not at module load: the DI
 * registry is installed by the composition root (and swapped per test), and
 * this module is imported long before either happens.
 */
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
