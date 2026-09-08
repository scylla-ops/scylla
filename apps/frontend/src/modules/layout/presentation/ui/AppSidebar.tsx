import * as React from 'react';
import { Building2 } from 'lucide-react';

import {
  NAV_SECTION_CARD_CLASS,
  NAV_SECTION_DIVIDER_CLASS,
  NavMain,
} from '@/modules/layout/presentation/ui/NavMain.tsx';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarRail,
} from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { NavUser } from '@/modules/layout/presentation/ui/NavUser.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { ContextSelector } from '@/modules/layout/presentation/ui/context-selector/ContextSelector.tsx';
import { OrganizationList } from '@/modules/features/organization';
import { AddOrganizationDialog } from '@/modules/features/organization';
import { CurrentContextDisplay } from '@/modules/layout/presentation/ui/context-selector/CurrentContextDisplay.tsx';
import { useContextStore } from '@platform/context';
import { useLingui } from '@lingui/react/macro';
import type { NavSection } from '@/modules/layout/presentation/structs/nav-section.struct.ts';
import type { NavEntry } from '@platform/routing';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { Permission } from '@platform/authz';
import { useAuthorization, useCan, } from '@platform/authz';
import { LanguageSelector } from '@/modules/layout/presentation/ui/LanguageSelector.tsx';
import { highlightIdForNav } from '@/modules/layout/presentation/hooks/use-whats-new.ts';

/**
 * Turns the entries the modules declared into rendered sidebar sections.
 *
 * The shell owns only what is genuinely shell here — the organization prefix,
 * the section cards, and hiding what the user may not reach. *Which* links exist
 * is each module's own `di/*.module.ts` declaration, the same one the router
 * reads, so a link can no longer point at a page that will deny you.
 */
const useNavSections = (
  entries: readonly NavEntry[],
  organizationHeader: React.ReactNode,
): { sections: NavSection[]; ready: boolean } => {
  const { t, i18n } = useLingui();

  const orgName = useContextStore(state => state.organization.name);
  const prefix = orgName ? `/${slugifyOrgName(orgName)}` : '';
  const { can, ready } = useAuthorization();

  const itemsOf = (section: NavEntry['section']) =>
    entries
      .filter(entry => entry.section === section)
      .map(entry => ({
        title: i18n._(entry.title),
        url: `${prefix}/${entry.url}`,
        icon: entry.icon,
        permission: entry.permission,
        highlightId: highlightIdForNav(entry.url),
      }));

  const sections: NavSection[] = [
    {
      title: t`Organization`,
      header: organizationHeader,
      items: itemsOf('organization'),
    },
    {
      title: t`System`,
      items: itemsOf('system'),
    },
  ];

  // Drop entries the current user can't access (in the current org), then any
  // section left empty. `can` defaults its target to the active org/project.
  // A section keeping a header (the org selector) stays: the user must be able
  // to switch organizations even with no readable link in this one.
  return {
    ready,
    sections: sections
      .map(section => ({
        ...section,
        items: section.items.filter(item => !item.permission || can(item.permission)),
      }))
      .filter(section => section.items.length > 0 || section.header),
  };
};

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  /** Declared by the modules, handed down by the router — see `navEntriesFor`. */
  navEntries: readonly NavEntry[];
}

export function AppSidebar({ navEntries, ...props }: AppSidebarProps) {
  const { t } = useLingui();
  const organization = useContextStore(state => state.organization);
  // Creating an organization isn't scoped to one — it's a system capability.
  const canCreateOrganization = useCan(Permission.CREATE_ORGANIZATION);

  // Sits at the top of the "Organization" card: the selector and the links it
  // scopes are one unit, not a floating panel above them.
  const organizationSelector = (
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
  );

  const { sections: navSections, ready } = useNavSections(navEntries, organizationSelector);

  return (
    <Sidebar variant={'inset'} collapsible='icon' {...props}>
      {/* Collapsed, the cards center themselves on the rail, so the side
          padding has to go or they no longer fit the 3rem width. */}
      <SidebarContent className='gap-3 px-2 py-2 group-data-[collapsible=icon]:px-0'>
        {ready ? (
          <NavMain sections={navSections} />
        ) : (
          // Permissions still loading — the selector stays put, entries become
          // skeletons, never a flash of links the user may not hold.
          <SidebarGroup className={NAV_SECTION_CARD_CLASS}>
            {organizationSelector}
            <div className={NAV_SECTION_DIVIDER_CLASS} />
            <div className='flex flex-col gap-1'>
              {Array.from({ length: 4 }).map((_, i) => (
                <div
                  key={i}
                  className='flex h-8 items-center gap-2 px-2 group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0'
                >
                  <Skeleton className='size-4 shrink-0 rounded' />
                  <Skeleton className='h-4 flex-1 group-data-[collapsible=icon]:hidden' />
                </div>
              ))}
            </div>
          </SidebarGroup>
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
