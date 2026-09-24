import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { MarketItem } from '@/modules/features/marketplace/domain/structs/market-item.struct.ts';

export class DefaultMarketplaceRepository implements MarketplaceRepository {
  // TODO: replace this fake data with a real data source.
  getMarketplace(): Promise<ScyllaResult<MarketItem[]>> {
    return Promise.resolve(
      ScyllaResult.try(
        () => [
          { provider: 'Corp', title: 'Title', descrption: 'Description' },
          { provider: 'Corp', title: 'Title', descrption: 'Description' },
          { provider: 'Epitech', title: 'TestFiltre', descrption: 'Description' },
        ],
        'Failed to load marketplace items.',
      ),
    );
  }
}
