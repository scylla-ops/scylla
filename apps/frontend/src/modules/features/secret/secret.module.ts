import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import type { SecretRemoteDataSource } from '@/modules/features/secret/infrastructure/repository/data-sources/secret-remote.data-source.ts';
import { DefaultSecretRepository } from '@/modules/features/secret/infrastructure/repository/default-secret.repository.ts';
import { grpcTransport } from '@platform/grpc';
import { GrpcSecretRemoteDataSource } from '@/modules/features/secret/infrastructure/data/grpc-credential-remote.data-source.ts';

const secretRemoteDataSource: SecretRemoteDataSource = new GrpcSecretRemoteDataSource(
  grpcTransport,
);
const secretRepository = new DefaultSecretRepository(secretRemoteDataSource);

export const SecretModule = {
  id: 'secret',
  domain: {
    secretRepository: secretRepository,
  },
  routes: {
    project: [
      {
        path: 'secrets',
        permission: Permission.LIST_SECRETS,
        breadcrumb: () => ({ label: msg`Secrets` }),
        page: () => import('./presentation/ui/Secret.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
