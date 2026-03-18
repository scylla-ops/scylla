import type { MarketItem } from '@/modules/features/marketplace/domain/models/MarketItem.ts';
import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export default interface MarketplaceRepository {
  getMarketplace(): Promise<ScyllaResult<MarketItem[]>>;
}
