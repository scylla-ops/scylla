import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { GrpcLoginRemoteDataSource } from '@/modules/features/login/infrastructure/data/remote/grpc-login-remote.data-source.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';
import { DefaultLoginRepository } from '@/modules/features/login/infrastructure/repository/default-login.repository.ts';
import { LoginUseCase } from '@/modules/features/login/domain/usecases/login.use-case.ts';
import { CoreModule } from '@core/di/core.module.ts';

const loginRemoteDataSource: LoginRemoteDataSource = new GrpcLoginRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const loginRepository: LoginRepository = new DefaultLoginRepository(loginRemoteDataSource);
const loginUseCase = new LoginUseCase(loginRepository);

export const LoginModule = {
  domain: { loginUseCase: loginUseCase },
};
