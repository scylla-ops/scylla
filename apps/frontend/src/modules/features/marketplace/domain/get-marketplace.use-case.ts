import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { MarketItemModel } from '@/modules/features/marketplace/domain/models/market-item.model.ts';

export class GetMarketplaceUseCase {
  constructor(private readonly repository: MarketplaceRepository) {}

  public async execute(): Promise<ScyllaResult<MarketItemModel[]>> {
    return this.repository.getMarketplace();
  }
}
