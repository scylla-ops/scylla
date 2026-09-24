import { msg } from '@lingui/core/macro';
import UsersRound from '@lucide/svelte/icons/users-round';
import { Permission } from '@platform/authz';
import type { ScyllaModule } from '@platform/routing';

export const MembershipModule = {
  id: 'membership',
  domain: {},
  routes: {
    organization: [
      {
        path: 'members',
        permission: Permission.LIST_ORGANIZATION_MEMBERS,
        breadcrumb: () => ({ label: msg`Members` }),
        page: () => import('./presentation/ui/OrganizationMembers/OrganizationMembers.page.svelte'),
        nav: { section: 'organization', title: msg`Members`, icon: UsersRound, order: 30 },
      },
    ],
    project: [
      {
        path: 'members',
        permission: Permission.LIST_PROJECT_MEMBERS,
        breadcrumb: () => ({ label: msg`Members` }),
        page: () => import('./presentation/ui/ProjectMembers/ProjectMembers.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
