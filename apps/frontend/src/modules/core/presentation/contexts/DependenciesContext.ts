import { createContext } from 'react';

import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';
import { LoginRemoteStoreImpl } from '@/modules/login/data/remote/LoginRemoteStoreImpl.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/login/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/login/domain/usecases/LoginUseCase.ts';
import type MarketplaceRepository from '@/modules/marketplace/domain/repository/MarketplaceRepository.ts';
import { MarketplaceRepositoryImpl } from '@/modules/marketplace/repository/MarketplaceRepositoryImpl.ts';
import { GetMarketplaceUseCase } from '@/modules/marketplace/domain/GetMarketplaceUseCase.ts';
import type { LoginMemoryStore } from '@/modules/login/repository/store/LoginMemoryStore.ts';
import { LoginMemoryStoreImpl } from '@/modules/login/data/memory/LoginMemoryStoreImpl.ts';
import { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import type { CoreMemoryStore } from '@core/repository/store/CoreMemoryStore.ts';
import { CoreMemoryStoreImpl } from '@core/data/memory/CoreMemoryStore.ts';

//todo: scind this by domain
class Dependencies {
  //core
  private coreMemoryStore: CoreMemoryStore = new CoreMemoryStoreImpl();
  private coreGrpcTransport: CoreGrpcTransport = new CoreGrpcTransport(this.coreMemoryStore);

  //login
  private loginRemoteStore: LoginRemoteStore = new LoginRemoteStoreImpl();
  private loginRepository: LoginRepository = new LoginRepositoryImpl(this.loginRemoteStore);
  public loginUseCase = new LoginUseCase(this.loginRepository);

  //marketplace
  private marketPlaceRepository: MarketplaceRepository = new MarketplaceRepositoryImpl();
  public getMarketplaceUseCase: GetMarketplaceUseCase = new GetMarketplaceUseCase(
    this.marketPlaceRepository,
  );
}

export const DependenciesContext = createContext<Dependencies | null>(new Dependencies());
