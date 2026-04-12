import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/MarketplaceRepository.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { MarketItem } from '@/modules/features/marketplace/domain/models/MarketItem.ts';

export class MarketplaceRepositoryImpl implements MarketplaceRepository {
  //TODO: replace this fake data with real api data by creating a data layer data-sources class (MarketplaceRemoteStore)
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
