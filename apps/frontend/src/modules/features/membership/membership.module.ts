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
  routes: [
    {
      mount: 'organization',
      path: 'members',
      permission: Permission.LIST_ORGANIZATION_MEMBERS,
      breadcrumb: () => ({ label: msg`Members` }),
      lazy: () => import('./presentation/ui/OrganizationMembers.page.svelte'),
    },
    {
      mount: 'project',
      path: 'members',
      permission: Permission.LIST_PROJECT_MEMBERS,
      breadcrumb: () => ({ label: msg`Members` }),
      lazy: () => import('./presentation/ui/ProjectMembers.page.svelte'),
    },
  ],
  nav: [
    {
      // Who belongs to the *current* organization — org-scoped, unlike the
      // system-wide directory under "System".
      section: 'organization',
      title: msg`Members`,
      url: 'members',
      icon: UsersRound,
      permission: Permission.LIST_ORGANIZATION_MEMBERS,
      order: 30,
    },
  ],
} satisfies ScyllaModule;
