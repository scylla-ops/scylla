import { createContext } from 'react';

import type { LoginStore } from '@/modules/login/repository/store/LoginStore.ts';
import { LoginStoreImpl } from '@/modules/login/data/remote/LoginStoreImpl.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/login/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/login/domain/usecases/LoginUseCase.ts';
import type MarketplaceRepository from '@/modules/marketplace/domain/repository/MarketplaceRepository.ts';
import { MarketplaceRepositoryImpl } from '@/modules/marketplace/repository/MarketplaceRepositoryImpl.ts';
import { GetMarketplaceUseCase } from '@/modules/marketplace/domain/GetMarketplaceUseCase.ts';

//todo: scind this by domain
class Dependencies {
  private loginStore: LoginStore = new LoginStoreImpl();
  private loginRepository: LoginRepository = new LoginRepositoryImpl(this.loginStore);
  public loginUseCase = new LoginUseCase(this.loginRepository);

  private marketPlaceRepository: MarketplaceRepository = new MarketplaceRepositoryImpl();
  public getMarketplaceUseCase: GetMarketplaceUseCase = new GetMarketplaceUseCase(
    this.marketPlaceRepository,
  );
}

export const DependenciesContext = createContext<Dependencies | null>(new Dependencies());
