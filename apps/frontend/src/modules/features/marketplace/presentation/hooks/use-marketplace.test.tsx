import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useMarketplace } from './use-marketplace';
import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import type { MarketItem } from '@/modules/features/marketplace/domain/structs/market-item.struct.ts';

const item = (overrides: Partial<MarketItem> = {}): MarketItem => ({
  provider: 'scylla',
  title: 'Hello world',
  descrption: 'a starter pipeline template',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<MarketplaceRepository> = {}) => {
  const getMarketplace =
    overrides.getMarketplace ?? vi.fn().mockResolvedValue(ScyllaResult.success([item()]));
  const repository: MarketplaceRepository = { getMarketplace };
  return { repository, getMarketplace };
};

const wrapperFor = (repository: MarketplaceRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={{ marketplace: { marketplaceRepository: repository } }}>
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return Wrapper;
};

describe('useMarketplace', () => {
  it('fetches the marketplace item list', async () => {
    const { repository, getMarketplace } = makeFakeRepository();
    const { result } = renderHook(() => useMarketplace(), { wrapper: wrapperFor(repository) });

    await waitFor(() => expect(result.current.data).toHaveLength(1));
    expect(getMarketplace).toHaveBeenCalled();
    expect(result.current.data?.[0].title).toBe('Hello world');
  });

  it('surfaces a repository error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      getMarketplace: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { result } = renderHook(() => useMarketplace(), { wrapper: wrapperFor(repository) });

    // The hook sets its own `retry: 1`, which overrides the client's
    // `retry: false` default for this query - so settling into the error
    // state takes one retry's backoff delay (~1s) longer than usual.
    await waitFor(() => expect(result.current.isError).toBe(true), { timeout: 3000 });
    expect(result.current.error).toBe(error);
  });
});
