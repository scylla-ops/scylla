import * as React from 'react';
import {
  Building2,
  ShoppingCartIcon,
  UsersIcon,
  WorkflowIcon,
  HardDriveIcon,
  ShieldIcon,
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
import type { NavSection } from '@/modules/layout/presentation/structs/nav-section.struct.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

const useNavSections = (): NavSection[] => {
  const orgName = useContextStore(state => state.organization.name);
  const prefix = orgName ? `/${slugifyOrgName(orgName)}` : '';
  const { can } = useAuthorization();

  const sections: NavSection[] = [
    {
      title: 'Main',
      items: [
        {
          title: 'Projects',
          url: `${prefix}/projects`,
          icon: WorkflowIcon,
          permission: Permission.LIST_PROJECTS_BY_ORGANIZATION,
        },
        {
          title: 'Agents',
          url: `${prefix}/agents`,
          icon: HardDriveIcon,
          permission: Permission.LIST_AGENTS,
        },
        {
          title: 'Marketplace',
          url: `${prefix}/marketplace`,
          icon: ShoppingCartIcon,
          permission: Permission.LIST_APPS_BY_ORGANIZATION,
        },
      ],
    },
    {
      title: 'Admin',
      items: [
        {
          title: 'Users',
          url: `${prefix}/users`,
          icon: UsersIcon,
          permission: Permission.LIST_USERS,
        },
        {
          title: 'Roles',
          url: `${prefix}/roles`,
          icon: ShieldIcon,
          permission: Permission.MANAGE_ROLES,
        },
      ],
    },
  ];

  // Drop entries the current user can't access (in the current org), then any
  // section left empty. `can` defaults its target to the active org/project.
  return sections
    .map(section => ({
      ...section,
      items: section.items.filter(item => !item.permission || can(item.permission)),
    }))
    .filter(section => section.items.length > 0);
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
