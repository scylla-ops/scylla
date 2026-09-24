import type { ScyllaModule } from '@platform/routing';
import { msg } from '@lingui/core/macro';
import UsersIcon from '@lucide/svelte/icons/users';
import { Permission } from '@platform/authz';
import { UserRemoteDataSourceImpl } from '@/modules/features/user/infrastructure/data/remote/user-remote.data-source.impl.ts';
import { grpcTransport } from '@platform/grpc';
import { DefaultUserRepository } from '@/modules/features/user/infrastructure/repository/default-user.repository.ts';

const dataSource = new UserRemoteDataSourceImpl(grpcTransport);
const repository = new DefaultUserRepository(dataSource);

export const UserModule = {
  id: 'user',
  domain: {
    /** Repository interface — the module's data surface. */
    userRepository: repository,
  },
  routes: {
    organization: [
      {
        path: 'users',
        permission: Permission.LIST_USERS,
        breadcrumb: () => ({ label: msg`Users` }),
        page: () => import('./presentation/ui/admin/UserAdmin.page.svelte'),
        nav: { section: 'system', title: msg`Users`, icon: UsersIcon, order: 10 },
      },
    ],
  },
} satisfies ScyllaModule;
