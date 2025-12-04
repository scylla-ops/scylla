import type MarketplaceRepository from '@/modules/marketplace/domain/repository/MarketplaceRepository.ts';
import { GetMarketplaceUseCase } from '@/modules/marketplace/domain/GetMarketplaceUseCase.ts';
import { MarketplaceRepositoryImpl } from '@/modules/marketplace/repository/MarketplaceRepositoryImpl.ts';

const marketPlaceRepository: MarketplaceRepository = new MarketplaceRepositoryImpl();
const getMarketplaceUseCase: GetMarketplaceUseCase = new GetMarketplaceUseCase(
  marketPlaceRepository,
);

export const MarketplaceModule = {
  domain: { getMarketplaceUseCase: getMarketplaceUseCase },
};
