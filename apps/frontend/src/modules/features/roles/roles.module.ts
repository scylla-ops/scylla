import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import { ShieldIcon } from 'lucide-react';
import { Permission } from '@platform/authz';
import { grpcTransport } from '@platform/grpc';
import { GrpcPermissionRemoteDataSource } from '@/modules/features/roles/infrastructure/data/grpc-permission-remote.data-source.ts';
import { DefaultPermissionRepository } from '@/modules/features/roles/infrastructure/repository/default-permission.repository.ts';
import { UpdateRoleUseCase } from '@/modules/features/roles/domain/use-cases/update-role.use-case.ts';

const dataSource = new GrpcPermissionRemoteDataSource(grpcTransport);
const repository = new DefaultPermissionRepository(dataSource);

export const RolesModule = {
  id: 'roles',
  domain: {
    /** Repository interface — the module's data surface. */
    permissionRepository: repository,
    updateRole: new UpdateRoleUseCase(repository),
  },
  routes: [
    {
      mount: 'organization',
      path: 'roles',
      permission: Permission.MANAGE_ROLES,
      breadcrumb: () => ({ label: msg`Roles` }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/Roles.page.tsx')).RolesPage,
      }),
    },
  ],
  nav: [
    {
      section: 'system',
      title: msg`Roles`,
      url: 'roles',
      icon: ShieldIcon,
      permission: Permission.MANAGE_ROLES,
      order: 20,
    },
  ],
} satisfies ScyllaModule;
