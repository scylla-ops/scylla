import * as React from 'react';
import { PlusIcon, ShoppingCartIcon, SquareTerminal } from 'lucide-react';

import { NavMain } from '@/modules/layout/presentation/ui/NavMain.tsx';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@shadcn/sidebar.tsx';
import { NavUser } from '@/modules/layout/presentation/ui/NavUser.tsx';
import { ContextSelector } from '@/modules/layout/presentation/ui/context-selector/ContextSelector.tsx';
import { CurrentOrganizationDisplay } from '@/modules/features/organization/presentation/ui/CurrentOrganizationDisplay.tsx';
import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';
import { AddOrganizationDialog } from '@/modules/features/organization/presentation/ui/AddOrganizationDialog.tsx';

// This is sample data.
const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
  navMain: [
    {
      title: 'Create pipeline',
      url: '/pipeline-creation',
      icon: PlusIcon,
    },
    {
      title: 'Pipelines',
      url: '/pipeline-dashboard',
      icon: SquareTerminal,
    },
    {
      title: 'Marketplace',
      url: '/marketplace',
      icon: ShoppingCartIcon,
    },
    {
      title: 'Settings',
      url: '/user-settings',
      icon: SquareTerminal,
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible='icon' {...props}>
      <SidebarHeader>
        <ContextSelector
          label={'Organization'}
          display={<CurrentOrganizationDisplay description={'Organization'} />}
          list={OrganizationList}
          addModal={AddOrganizationDialog}
        />
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
