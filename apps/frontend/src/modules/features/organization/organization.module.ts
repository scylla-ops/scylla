import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import DefaultOrganizationRepository from '@/modules/features/organization/infrastructure/repository/default-organization.repository.ts';
import GrpcOrganizationRemoteDataSource from '@/modules/features/organization/infrastructure/data/grpc-organization-remote.data-source.ts';
import { grpcTransport } from '@platform/grpc';

const organizationRemoteDataSource = new GrpcOrganizationRemoteDataSource(grpcTransport);

const organizationRepository = new DefaultOrganizationRepository(organizationRemoteDataSource);

export const OrganizationModule = {
  id: 'organization',
  domain: {
    /** Repository interface — the module's data surface. */
    organizationRepository: organizationRepository,
  },
  routes: {
    organization: [
      {
        // Under the user directory that `user` owns: this page is here because
        // it renders the organizations panel.
        path: 'users/:userId',
        breadcrumb: ({ userId }) => ({ label: msg`User`, highlight: userId, detail: msg`Detail` }),
        page: () => import('./presentation/ui/UserSettingsRoute.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
