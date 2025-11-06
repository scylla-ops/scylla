import type MarketplaceRepository from '@/modules/marketplace/domain/repository/MarketplaceRepository.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import type { MarketItem } from '@/modules/marketplace/domain/models/MarketItem.ts';

export class GetMarketplaceUseCase {
  constructor(private readonly repository: MarketplaceRepository) {}

  public async execute(): Promise<ScyllaResult<MarketItem[]>> {
    return this.repository.getMarketplace();
  }
}
