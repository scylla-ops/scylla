import { msg } from '@lingui/core/macro';
import UsersRound from '@lucide/svelte/icons/users-round';
import { Permission } from '@platform/authz';
import type { ScyllaModule } from '@platform/routing';

/**
 * Members of an organization or of a project. Both pages read other modules'
 * data, so like `dashboard` this module contributes routes without a domain.
 */
export const MembershipModule = {
  id: 'membership',
  domain: {},
  routes: {
    organization: [
      {
        path: 'members',
        permission: Permission.LIST_ORGANIZATION_MEMBERS,
        breadcrumb: () => ({ label: msg`Members` }),
        page: () => import('./presentation/ui/OrganizationMembers.page.svelte'),
        // Who belongs to the *current* organization — org-scoped, unlike the
        // system-wide directory under "System".
        nav: { section: 'organization', title: msg`Members`, icon: UsersRound, order: 30 },
      },
    ],
    project: [
      {
        path: 'members',
        permission: Permission.LIST_PROJECT_MEMBERS,
        breadcrumb: () => ({ label: msg`Members` }),
        page: () => import('./presentation/ui/ProjectMembers.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
