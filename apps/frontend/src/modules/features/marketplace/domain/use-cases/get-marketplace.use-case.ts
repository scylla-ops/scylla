import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { MarketItem } from '@/modules/features/marketplace/domain/structs/market-item.struct.ts';

export class GetMarketplaceUseCase {
  constructor(private readonly repository: MarketplaceRepository) {}

  public async execute(): Promise<ScyllaResult<MarketItem[]>> {
    return this.repository.getMarketplace();
  }
}
