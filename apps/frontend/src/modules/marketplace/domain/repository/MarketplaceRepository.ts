import type { MarketItem } from '@/modules/marketplace/domain/models/MarketItem.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export default interface MarketplaceRepository {
  getMarketplace(): Promise<ScyllaResult<MarketItem[]>>;
}
