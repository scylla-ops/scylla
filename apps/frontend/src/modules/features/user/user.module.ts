import type { ScyllaModule } from '@platform/routing/scylla-module.struct.ts';
import { msg } from '@lingui/core/macro';
import { UsersIcon } from 'lucide-react';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import { UserRemoteDataSourceImpl } from '@/modules/features/user/infrastructure/data/remote/user-remote.data-source.impl.ts';
import { grpcTransport } from '@platform/grpc/index.ts';
import { DefaultUserRepository } from '@/modules/features/user/infrastructure/repository/default-user.repository.ts';

const dataSource = new UserRemoteDataSourceImpl(grpcTransport);
const repository = new DefaultUserRepository(dataSource);

export const UserModule = {
  id: 'user',
  domain: {
    /** Repository interface — the module's data surface. */
    userRepository: repository,
  },
  routes: [
    {
      mount: 'organization',
      path: 'users',
      breadcrumb: () => ({ label: msg`Users` }),
      children: [
        {
          mount: 'organization',
          index: true,
          permission: Permission.LIST_USERS,
          lazy: async () => ({
            Component: (await import('./presentation/ui/admin/UserAdmin.page.tsx')).UserAdminPage,
          }),
        },
      ],
    },
  ],
  nav: [
    {
      section: 'system',
      title: msg`Users`,
      url: 'users',
      icon: UsersIcon,
      permission: Permission.LIST_USERS,
      order: 10,
    },
  ],
} satisfies ScyllaModule;
