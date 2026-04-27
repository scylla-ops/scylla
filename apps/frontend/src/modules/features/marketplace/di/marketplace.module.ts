import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/marketplace.repository.ts';
import { GetMarketplaceUseCase } from '@/modules/features/marketplace/domain/get-marketplace.use-case.ts';
import { DefaultMarketplaceRepository } from '@/modules/features/marketplace/infrastructure/repository/default-marketplace.repository.ts';

const marketPlaceRepository: MarketplaceRepository = new DefaultMarketplaceRepository();
const getMarketplaceUseCase: GetMarketplaceUseCase = new GetMarketplaceUseCase(
  marketPlaceRepository,
);

export const MarketplaceModule = {
  domain: { getMarketplaceUseCase: getMarketplaceUseCase },
};
