import type { ScyllaModule } from '@platform/routing';
import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { GrpcLoginRemoteDataSource } from '@/modules/features/login/infrastructure/data/remote/grpc-login-remote.data-source.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';
import { DefaultLoginRepository } from '@/modules/features/login/infrastructure/repository/default-login.repository.ts';
import { grpcTransport } from '@platform/grpc';

const loginRemoteDataSource: LoginRemoteDataSource = new GrpcLoginRemoteDataSource(grpcTransport);
const loginRepository: LoginRepository = new DefaultLoginRepository(loginRemoteDataSource);

export const LoginModule = {
  id: 'login',
  domain: {
    /** Repository interface — the module's data surface. */
    loginRepository: loginRepository,
  },
  routes: {
    public: [{ path: 'login', page: () => import('./presentation/ui/Login/Login.page.svelte') }],
  },
} satisfies ScyllaModule;
