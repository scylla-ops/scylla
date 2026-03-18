import type MarketplaceRepository from '@/modules/features/marketplace/domain/repository/MarketplaceRepository.ts';
import { GetMarketplaceUseCase } from '@/modules/features/marketplace/domain/GetMarketplaceUseCase.ts';
import { MarketplaceRepositoryImpl } from '@/modules/features/marketplace/infrastructure/repository/MarketplaceRepositoryImpl.ts';

const marketPlaceRepository: MarketplaceRepository = new MarketplaceRepositoryImpl();
const getMarketplaceUseCase: GetMarketplaceUseCase = new GetMarketplaceUseCase(
  marketPlaceRepository,
);

export const MarketplaceModule = {
  domain: { getMarketplaceUseCase: getMarketplaceUseCase },
};
