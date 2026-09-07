import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import DefaultOrganizationRepository from '@/modules/features/organization/infrastructure/repository/default-organization.repository.ts';
import GrpcOrganizationRemoteDataSource from '@/modules/features/organization/infrastructure/data/grpc-organization-remote.data-source.ts';
import { grpcTransport } from '@platform/grpc/index.ts';

const organizationRemoteDataSource = new GrpcOrganizationRemoteDataSource(
  grpcTransport,
);

const organizationRepository = new DefaultOrganizationRepository(organizationRemoteDataSource);

export const OrganizationModule = {
  id: 'organization',
  domain: {
    /** Repository interface — the module's data surface. */
    organizationRepository: organizationRepository,
  },
  routes: [
    {
      // The user directory belongs to `user`; this leaf is here because the
      // settings page renders the organizations panel. Both halves are merged
      // onto one `users` parent by the route composer.
      mount: 'organization',
      path: 'users',
      children: [
        {
          mount: 'organization',
          path: ':userId',
          breadcrumb: ({ userId }) => ({
            label: msg`User`,
            highlight: userId,
            detail: msg`Detail`,
          }),
          lazy: async () => ({
            Component: (await import('./presentation/ui/UserSettingsRoute.tsx')).UserSettingsRoute,
          }),
        },
      ],
    },
  ],
} satisfies ScyllaModule;
