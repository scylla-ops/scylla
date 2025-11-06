import type { MarketItem } from '@/modules/marketplace/domain/models/MarketItem.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export default interface MarketplaceRepository {
  getMarketplace(): Promise<ScyllaResult<MarketItem[]>>;
}
