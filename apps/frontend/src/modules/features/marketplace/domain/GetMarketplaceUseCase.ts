import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/MarketplaceRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { MarketItem } from '@/modules/features/marketplace/domain/models/MarketItem.ts';

export class GetMarketplaceUseCase {
  constructor(private readonly repository: MarketplaceRepository) {}

  public async execute(): Promise<ScyllaResult<MarketItem[]>> {
    return this.repository.getMarketplace();
  }
}
