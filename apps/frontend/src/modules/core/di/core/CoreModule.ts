//core
import type { CoreMemoryStore } from '@core/repository/store/CoreMemoryStore.ts';
import { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';
import { CoreMemoryStoreImpl } from '@core/data/memory/CoreMemoryStore.ts';
import { CoreRepositoryImpl } from '@core/repository/CoreRepositoryImpl.ts';
import { SetTokenUseCase } from '@core/domain/usecases/SetTokenUseCase.ts';
import { GetTokenUseCase } from '@core/domain/usecases/GetTokenUseCase.ts';

const coreMemoryStore: CoreMemoryStore = new CoreMemoryStoreImpl();
const coreGrpcTransport: CoreGrpcTransport = new CoreGrpcTransport(coreMemoryStore);
const coreRepository = new CoreRepositoryImpl(coreMemoryStore);

const setTokenUseCase = new SetTokenUseCase(coreRepository);
const getTokenUseCase = new GetTokenUseCase(coreRepository);

export const CoreModule = {
  data: { coreGrpcTransport: coreGrpcTransport },
  domain: { setTokenUseCase: setTokenUseCase, getTokenUseCase: getTokenUseCase },
};
