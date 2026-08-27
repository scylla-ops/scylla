import * as React from 'react';
import {
  Building2,
  ShoppingCartIcon,
  UsersIcon,
  UsersRound,
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
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { ContextSelector } from '@/modules/layout/presentation/ui/context-selector/ContextSelector.tsx';
import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';
import { AddOrganizationDialog } from '@/modules/features/organization/presentation/ui/AddOrganizationDialog.tsx';
import { CurrentContextDisplay } from '@/modules/layout/presentation/ui/context-selector/CurrentContextDisplay.tsx';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useLingui } from '@lingui/react/macro';
import type { NavSection } from '@/modules/layout/presentation/structs/nav-section.struct.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  useAuthorization,
  useCan,
} from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { LanguageSelector } from '@/modules/layout/presentation/ui/LanguageSelector.tsx';

const useNavSections = (): { sections: NavSection[]; ready: boolean } => {
  const { t } = useLingui();

  const orgName = useContextStore(state => state.organization.name);
  const prefix = orgName ? `/${slugifyOrgName(orgName)}` : '';
  const { can, ready } = useAuthorization();

  const sections: NavSection[] = [
    {
      title: t`Organization`,
      items: [
        {
          title: t`Projects`,
          url: `${prefix}/projects`,
          icon: WorkflowIcon,
          permission: Permission.READ_ORGANIZATION,
        },
        {
          // Who belongs to the *current* organization — org-scoped, unlike the
          // system-wide directory below it.
          title: t`Members`,
          url: `${prefix}/members`,
          icon: UsersRound,
          permission: Permission.LIST_ORGANIZATION_MEMBERS,
        },
        {
          title: t`Agents`,
          url: `${prefix}/agents`,
          icon: HardDriveIcon,
          permission: Permission.LIST_AGENTS,
        },
        {
          title: t`Marketplace`,
          url: `${prefix}/marketplace`,
          icon: ShoppingCartIcon,
          permission: Permission.LIST_APPS_BY_ORGANIZATION,
        },
      ],
    },
    {
      title: t`System`,
      items: [
        {
          title: t`Users`,
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
  return {
    ready,
    sections: sections
      .map(section => ({
        ...section,
        items: section.items.filter(item => !item.permission || can(item.permission)),
      }))
      .filter(section => section.items.length > 0),
  };
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { t } = useLingui();
  const organization = useContextStore(state => state.organization);
  const { sections: navSections, ready } = useNavSections();
  // Creating an organization isn't scoped to one — it's a system capability.
  const canCreateOrganization = useCan(Permission.CREATE_ORGANIZATION);

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
          canAdd={canCreateOrganization}
        />
      </SidebarHeader>
      <SidebarContent>
        {ready ? (
          <NavMain sections={navSections} />
        ) : (
          // Permissions still loading — skeleton entries, never a flash of
          // links the user may not hold.
          <div className='flex flex-col gap-2 px-3 py-4'>
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className='flex items-center gap-2'>
                <Skeleton className='size-4 rounded' />
                <Skeleton className='h-4 flex-1' />
              </div>
            ))}
          </div>
        )}
      </SidebarContent>
      <SidebarFooter className='flex flex-col gap-2'>
        <LanguageSelector />
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
