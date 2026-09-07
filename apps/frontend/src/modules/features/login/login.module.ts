import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { GrpcLoginRemoteDataSource } from '@/modules/features/login/infrastructure/data/remote/grpc-login-remote.data-source.ts';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';
import { DefaultLoginRepository } from '@/modules/features/login/infrastructure/repository/default-login.repository.ts';
import { grpcTransport } from '@platform/grpc/index.ts';

const loginRemoteDataSource: LoginRemoteDataSource = new GrpcLoginRemoteDataSource(
  grpcTransport,
);
const loginRepository: LoginRepository = new DefaultLoginRepository(loginRemoteDataSource);

export const LoginModule = {
  id: 'login',
  domain: {
    /** Repository interface — the module's data surface. */
    loginRepository: loginRepository,
  },
  routes: [
    {
      mount: 'public',
      path: '/login',
      lazy: async () => ({
        Component: (await import('./presentation/ui/Login.page.tsx')).LoginPage,
      }),
    },
  ],
} satisfies ScyllaModule;
