import type { ScyllaModule } from '@platform/routing';
import { AppsRemoteDataSourceImpl } from '@/modules/features/apps/infrastructure/data/apps-remote.data-source.ts';
import { grpcTransport } from '@platform/grpc';
import { DefaultAppsRepository } from '@/modules/features/apps/infrastructure/repository/default-apps.repository.ts';

const dataSource = new AppsRemoteDataSourceImpl(grpcTransport);
const repository = new DefaultAppsRepository(dataSource);

export const AppsModule = {
  id: 'apps',
  domain: {
    /** Repository interface — the module's data surface. */
    appsRepository: repository,
  },

} satisfies ScyllaModule;
