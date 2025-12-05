import type MarketplaceRepository from '@/modules/marketplace/domain/repository/MarketplaceRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { MarketItem } from '@/modules/marketplace/domain/models/MarketItem.ts';

export class MarketplaceRepositoryImpl implements MarketplaceRepository {
  //TODO: replace this fake data with real api data by creating a data layer stores class (MarketplaceRemoteStore)
  getMarketplace(): Promise<ScyllaResult<MarketItem[]>> {
    return Promise.resolve({
      ok: true,
      value: [
        { provider: 'Corp', title: 'Title', descrption: 'Description' },
        { provider: 'Corp', title: 'Title', descrption: 'Description' },
        { provider: 'Epitech', title: 'TestFiltre', descrption: 'Description' },
      ],
    });
  }
}
