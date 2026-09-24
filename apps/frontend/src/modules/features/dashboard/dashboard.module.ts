import { msg } from '@lingui/core/macro';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import { Permission } from '@platform/authz';
import type { ScyllaModule } from '@platform/routing';

export const DashboardModule = {
  id: 'dashboard',
  domain: {},
  routes: {
    organization: [
      {
        path: 'dashboard',
        // The overview reads the organization; every organization redirect lands here.
        permission: Permission.READ_ORGANIZATION,
        breadcrumb: () => ({ label: msg`Dashboard` }),
        page: () => import('./presentation/ui/Dashboard/Dashboard.page.svelte'),
        nav: { section: 'organization', title: msg`Dashboard`, icon: LayoutDashboard, order: 10 },
      },
    ],
  },
} satisfies ScyllaModule;
