import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/LoginRemoteDataSource.ts';
import { LoginRemoteDataSourceImpl } from '@/modules/features/login/infrastructure/data/remote/LoginRemoteDataSourceImpl.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/LoginRepository.ts';
import { LoginRepositoryImpl } from '@/modules/features/login/infrastructure/repository/LoginRepositoryImpl.ts';
import { LoginUseCase } from '@/modules/features/login/domain/usecases/LoginUseCase.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';

const loginRemoteDataSource: LoginRemoteDataSource = new LoginRemoteDataSourceImpl(
  CoreModule.data.coreGrpcTransport,
);
const loginRepository: LoginRepository = new LoginRepositoryImpl(loginRemoteDataSource);
const loginUseCase = new LoginUseCase(loginRepository);

export const LoginModule = {
  domain: { loginUseCase: loginUseCase },
};
