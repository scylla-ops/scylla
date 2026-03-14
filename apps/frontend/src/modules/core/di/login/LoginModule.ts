import type { LoginRemoteDataSource } from '@/modules/login/repository/data-sources/LoginRemoteDataSource.ts';
import { LoginRemoteDataSourceImpl } from '@/modules/login/data/remote/LoginRemoteDataSourceImpl.ts';
import type { LoginRepository } from '@/modules/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/login/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/login/domain/usecases/LoginUseCase.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';

const loginRemoteDataSource: LoginRemoteDataSource = new LoginRemoteDataSourceImpl(
  CoreModule.data.coreGrpcTransport,
);
const loginRepository: LoginRepository = new LoginRepositoryImpl(loginRemoteDataSource);
const loginUseCase = new LoginUseCase(loginRepository);

export const LoginModule = {
  domain: { loginUseCase: loginUseCase },
};
