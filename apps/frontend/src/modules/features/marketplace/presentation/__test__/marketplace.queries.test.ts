// @vitest-environment node
import { describe, it, expect, vi, afterEach } from 'vitest';
import { setDependencyRegistry } from '@platform/di';
import { runQueryFn } from '@/test/queries.ts';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import type MarketplaceRepository from '../../domain/repository/marketplace.repository.ts';
import type { MarketItem } from '../../domain/structs/market-item.struct.ts';
import { MARKETPLACE_QUERY_KEY, marketplaceQueries } from '../marketplace.queries.ts';

const item = (overrides: Partial<MarketItem> = {}): MarketItem => ({
  provider: 'scylla',
  title: 'Hello world',
  descrption: 'a starter pipeline template',
  ...overrides,
});

const withRepository = (overrides: Partial<MarketplaceRepository> = {}) => {
  const getMarketplace =
    overrides.getMarketplace ?? vi.fn().mockResolvedValue(ScyllaResult.success([item()]));
  setDependencyRegistry({ marketplace: { marketplaceRepository: { getMarketplace } } });
  return { getMarketplace };
};

afterEach(() => setDependencyRegistry(null));

describe('marketplaceQueries.list', () => {
  it('fetches the catalog through the injected repository', async () => {
    const { getMarketplace } = withRepository();

    const items = await runQueryFn(marketplaceQueries.list());

    expect(getMarketplace).toHaveBeenCalled();
    expect(items).toEqual([item()]);
  });

  it('lets a repository error through, for the query to own', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    withRepository({ getMarketplace: vi.fn().mockResolvedValue(ScyllaResult.error(error)) });

    await expect(runQueryFn(marketplaceQueries.list())).rejects.toBe(error);
  });

  it('caches under the key the factory publishes, so an invalidation can find it', () => {
    expect(marketplaceQueries.list().queryKey).toEqual(MARKETPLACE_QUERY_KEY());
  });

  it('resolves the repository per call, not at import time', async () => {
    // Resolved per call: the registry is installed after this module loads.
    const first = withRepository();
    await runQueryFn(marketplaceQueries.list());

    const second = withRepository({
      getMarketplace: vi.fn().mockResolvedValue(ScyllaResult.success([item({ title: 'Other' })])),
    });
    const items = await runQueryFn(marketplaceQueries.list());

    expect(first.getMarketplace).toHaveBeenCalledTimes(1);
    expect(second.getMarketplace).toHaveBeenCalledTimes(1);
    expect(items[0].title).toBe('Other');
  });
});
