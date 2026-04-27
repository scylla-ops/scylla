import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { MarketItemModel } from '@/modules/features/marketplace/domain/models/market-item.model.ts';

export class DefaultMarketplaceRepository implements MarketplaceRepository {
  //TODO: replace this fake data with real api data by creating a data layer data-sources class (MarketplaceRemoteStore)
  getMarketplace(): Promise<ScyllaResult<MarketItemModel[]>> {
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
