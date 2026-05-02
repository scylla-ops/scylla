import type { MarketItemModel } from '@/modules/features/marketplace/domain/models/market-item.model.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export default interface MarketplaceRepository {
  getMarketplace(): Promise<ScyllaResult<MarketItemModel[]>>;
}
