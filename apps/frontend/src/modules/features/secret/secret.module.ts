import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import type { SecretRemoteDataSource } from '@/modules/features/secret/infrastructure/repository/data-sources/secret-remote.data-source.ts';
import { DefaultSecretRepository } from '@/modules/features/secret/infrastructure/repository/default-secret.repository.ts';
import { grpcTransport } from '@platform/grpc/index.ts';
import { GrpcSecretRemoteDataSource } from '@/modules/features/secret/infrastructure/data/grpc-credential-remote.data-source.ts';

const secretRemoteDataSource: SecretRemoteDataSource = new GrpcSecretRemoteDataSource(
  grpcTransport,
);
const secretRepository = new DefaultSecretRepository(secretRemoteDataSource);

export const SecretModule = {
  id: 'secret',
  domain: {
    /** Repository interface — the module's data surface. */
    secretRepository: secretRepository,
  },
  routes: [
    {
      mount: 'project',
      path: 'secrets',
      permission: Permission.LIST_SECRETS,
      breadcrumb: () => ({ label: msg`Secrets` }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/Secret.page.tsx')).SecretPage,
      }),
    },
  ],
} satisfies ScyllaModule;
