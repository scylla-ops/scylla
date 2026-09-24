import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { Permission } from '@platform/authz';
import { grpcTransport } from '@platform/grpc';
import type { TriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/repository/data-sources/triggers-remote.data-source.ts';
import { GrpcTriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/data/remote/grpc-triggers-remote.data-source.ts';
import { DefaultTriggersRepository } from '@/modules/features/triggers/infrastructure/repository/default-triggers.repository.ts';

const triggersRemoteDataSource: TriggersRemoteDataSource = new GrpcTriggersRemoteDataSource(
  grpcTransport,
);
const triggersRepository = new DefaultTriggersRepository(triggersRemoteDataSource);

export const TriggersModule = {
  id: 'triggers',
  domain: {
    triggersRepository: triggersRepository,
  },
  routes: {
    project: [
      {
        path: 'pipelines/:pipelineId/triggers',
        permission: Permission.MANAGE_TRIGGERS,
        breadcrumb: ({ pipelineName }) => ({
          label: msg`Pipeline`,
          highlight: pipelineName,
          detail: msg`Triggers`,
        }),
        page: () => import('./presentation/ui/Triggers/Triggers.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
