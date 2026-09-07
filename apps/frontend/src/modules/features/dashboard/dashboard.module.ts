import { msg } from '@lingui/core/macro';
import { LayoutDashboard } from 'lucide-react';
import { Permission } from '@platform/authz';
import type { ScyllaModule } from '@platform/routing';

/**
 * The organization overview. It owns no data of its own — it composes other
 * modules' hooks — so its `domain` is empty and it contributes only a route.
 */
export const DashboardModule = {
  id: 'dashboard',
  domain: {},
  routes: [
    {
      mount: 'organization',
      path: 'dashboard',
      // Same gate as the projects list: the overview is a read of the
      // organization, and it is where every org-level redirect lands.
      permission: Permission.READ_ORGANIZATION,
      breadcrumb: () => ({ label: msg`Dashboard` }),
      lazy: async () => ({
        Component: (await import('./presentation/ui/Dashboard.page.tsx')).DashboardPage,
      }),
    },
  ],
  nav: [
    {
      section: 'organization',
      title: msg`Dashboard`,
      url: 'dashboard',
      icon: LayoutDashboard,
      permission: Permission.READ_ORGANIZATION,
      order: 10,
    },
  ],
} satisfies ScyllaModule;
