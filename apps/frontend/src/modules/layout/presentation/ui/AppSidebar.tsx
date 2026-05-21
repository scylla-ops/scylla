import * as React from 'react';
import {
  Building2,
  ShoppingCartIcon,
  UsersIcon,
  WorkflowIcon,
  HardDriveIcon,
  FolderKanbanIcon,
  CpuIcon,
  ServerIcon,
} from 'lucide-react';

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
import { slugifyOrgName } from '@shared/utils/slug.ts';

const useNavSections = (): NavSectionModel[] => {
  const orgName = useContextStore(state => state.organization.name);
  const prefix = orgName ? `/${slugifyOrgName(orgName)}` : '';

  return [
    {
      // Organization scope (Org → Project → Pipeline → Job; App ∈ Org).
      title: 'Organization',
      items: [
        {
          title: 'Workspace',
          icon: WorkflowIcon,
          items: [
            {
              title: 'Projects',
              url: `${prefix}/projects`,
              icon: FolderKanbanIcon,
            },
            {
              title: 'Marketplace',
              url: `${prefix}/marketplace`,
              icon: ShoppingCartIcon,
            },
          ],
        },
        {
          title: 'Service',
          icon: ServerIcon,
          items: [
            {
              title: 'Apps',
              url: `${prefix}/apps`,
              icon: HardDriveIcon,
            },
            {
              title: 'Workers',
              url: `${prefix}/workers`,
              icon: CpuIcon,
            },
          ],
        },
      ],
    },
    {
      // Global administration (system scope).
      title: 'Administration',
      items: [
        {
          title: 'Users',
          url: `${prefix}/users-admin`,
          icon: UsersIcon,
        },
      ],
    },
  ];
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { t } = useLingui();
  const organization = useContextStore(state => state.organization);
  const navSections = useNavSections();

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
