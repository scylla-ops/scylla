import * as React from 'react';
import { Building2, ShoppingCartIcon, UsersIcon, WorkflowIcon, HardDriveIcon } from 'lucide-react';

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
import { CurrentContextDisplay } from '@/modules/layout/presentation/ui/context-selector/CurrentContextDisplay.tsx';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useLingui } from '@lingui/react/macro';
import type { NavSectionModel } from '@/modules/layout/presentation/models/nav-section.model.ts';

const navSections: NavSectionModel[] = [
  {
    title: 'Main',
    items: [
      {
        title: 'Projects',
        url: '/projects',
        icon: WorkflowIcon,
      },
      {
        title: 'Marketplace',
        url: '/marketplace',
        icon: ShoppingCartIcon,
      },
      {
        title: 'Workers',
        url: '/workers',
        icon: HardDriveIcon,
      },
    ],
  },
  {
    title: 'Admin',
    items: [
      {
        title: 'Users',
        url: '/users-admin',
        icon: UsersIcon,
      },
    ],
  },
];

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { t } = useLingui();
  const organization = useContextStore(state => state.organization);

  return (
    <Sidebar variant={'inset'} collapsible='icon' {...props}>
      <SidebarHeader>
        <ContextSelector
          label={t`Organization`}
          display={
            <CurrentContextDisplay
              name={organization.name || t`Select Organization`}
              description={t`Organization`}
              icon={Building2}
            />
          }
          list={OrganizationList}
          addModal={AddOrganizationDialog}
        />
      </SidebarHeader>
      <SidebarContent>
        <NavMain sections={navSections} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
