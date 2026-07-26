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
import type { NavSection } from '@/modules/layout/presentation/structs/nav-section.struct.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { LanguagesIcon } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/modules/shared/presentation/ui/shadcn/dropdown-menu.tsx';
import { Button } from '@/modules/shared/presentation/ui/shadcn/button.tsx';
import {
  getCurrentLocale,
  setAppLocale,
  type SupportedLocale,
} from '@shared/presentation/utils/i18n.ts';

const useNavSections = (): NavSection[] => {
  const { t } = useLingui();
  const orgName = useContextStore(state => state.organization.name);
  const prefix = orgName ? `/${slugifyOrgName(orgName)}` : '';

  return [
    {
      title: t`Main`,
      items: [
        {
          title: t`Projects`,
          url: `${prefix}/projects`,
          icon: WorkflowIcon,
        },
        {
          title: t`Agents`,
          url: `${prefix}/agents`,
          icon: HardDriveIcon,
        },
        {
          title: t`Marketplace`,
          url: `${prefix}/marketplace`,
          icon: ShoppingCartIcon,
        },
      ],
    },
    {
      title: t`Admin`,
      items: [
        {
          title: t`Users`,
          url: `${prefix}/users`,
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
  const [currentLocale, setCurrentLocale] = React.useState<SupportedLocale>(() =>
    getCurrentLocale(),
  );

  const handleLocaleChange = (locale: SupportedLocale) => {
    setCurrentLocale(locale);
    setAppLocale(locale);
  };

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
      <SidebarFooter className='flex flex-col gap-2'>
        <div className='flex items-center justify-between rounded-lg border border-sidebar-border/70 bg-sidebar/40 px-3 py-2'>
          <div className='flex items-center gap-2 text-sm text-muted-foreground'>
            <LanguagesIcon className='size-4' />
            <span>{t`Language`}</span>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='sm' className='h-8 px-2'>
                {currentLocale === 'fr' ? 'FR' : 'EN'}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuItem onSelect={() => handleLocaleChange('en')}>English</DropdownMenuItem>
              <DropdownMenuItem onSelect={() => handleLocaleChange('fr')}>
                Français
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
