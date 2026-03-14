import * as React from 'react';
import { SquareTerminal } from 'lucide-react';

import { NavMain } from '@/modules/layout/presentation/NavMain.tsx';
import { OrganisationSwitcher } from '@/modules/features/organization/presentation/OrganisationSwitcher.tsx';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@shadcn/sidebar.tsx';
import { NavUser } from '@/modules/layout/presentation/NavUser.tsx';

// This is sample data.
const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
  navMain: [
    {
      title: 'Pipelines',
      url: '#',
      icon: SquareTerminal,
      isActive: true,
      items: [
        {
          title: 'Create pipeline',
          url: '/pipeline-creation',
        },
        {
          title: 'Pipelines',
          url: '/pipeline-dashboard',
        },
        {
          title: 'Marketplace',
          url: '/marketplace',
        },
      ],
    },

    {
      title: 'Settings',
      url: '#',
      icon: SquareTerminal,
      isActive: true,
      items: [
        {
          title: 'Users',
          url: '/user-settings',
        },
      ],
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible='icon' {...props}>
      <SidebarHeader>
        <OrganisationSwitcher />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
