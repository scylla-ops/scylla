import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';
import { LoginRemoteStoreImpl } from '@/modules/login/data/remote/LoginRemoteStoreImpl.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/login/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/login/domain/usecases/LoginUseCase.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';

const loginRemoteStore: LoginRemoteStore = new LoginRemoteStoreImpl(
  CoreModule.data.coreGrpcTransport,
);
const loginRepository: LoginRepository = new LoginRepositoryImpl(loginRemoteStore);
const loginUseCase = new LoginUseCase(loginRepository);

export const LoginModule = {
  domain: { loginUseCase: loginUseCase },
};
