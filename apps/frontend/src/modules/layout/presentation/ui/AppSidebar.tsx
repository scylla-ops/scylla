import * as React from 'react';
import { Building2, ProjectorIcon, ShoppingCartIcon, WorkflowIcon } from 'lucide-react';

import { NavMain } from '@/modules/layout/presentation/ui/NavMain.tsx';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { NavUser } from '@/modules/layout/presentation/ui/NavUser.tsx';
import { ContextSelector } from '@/modules/layout/presentation/ui/context-selector/ContextSelector.tsx';
import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';
import { AddOrganizationDialog } from '@/modules/features/organization/presentation/ui/AddOrganizationDialog.tsx';
import ProjectList from '@/modules/features/project/presentation/ui/ProjectList.tsx';
import AddProjectDialog from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { CurrentContextDisplay } from '@/modules/layout/presentation/ui/context-selector/CurrentContextDisplay.tsx';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';

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
      url: '/pipeline-dashboard',
      icon: WorkflowIcon,
    },
    {
      title: 'Marketplace',
      url: '/marketplace',
      icon: ShoppingCartIcon,
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const organization = useContextStore(state => state.organization);
  const project = useContextStore(state => state.project);

  return (
    <Sidebar collapsible='icon' {...props}>
      <SidebarHeader>
        <ContextSelector
          label={'Organization'}
          display={
            <CurrentContextDisplay
              name={organization.name || 'Select Organization'}
              description={'Organization'}
              icon={Building2}
            />
          }
          list={OrganizationList}
          addModal={AddOrganizationDialog}
        />
        <ContextSelector
          label={'Project'}
          display={
            <CurrentContextDisplay
              variant={'secondary'}
              name={project.name || 'Select Project'}
              description={'Project'}
              icon={ProjectorIcon}
            />
          }
          list={ProjectList}
          addModal={AddProjectDialog}
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
