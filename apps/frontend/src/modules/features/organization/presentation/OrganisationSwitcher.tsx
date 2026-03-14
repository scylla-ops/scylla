import * as React from 'react';
import { ChevronsUpDown, Plus, Building2 } from 'lucide-react';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem, useSidebar } from '@shadcn/sidebar.tsx';
import { AddOrganizationDialog } from '@/modules/features/organization/presentation/AddOrganizationDialog.tsx';
import { useOrganization } from '@/modules/features/organization/presentation/hooks/useOrganization.ts';
import type { OrganizationResponse } from '@/generated/organization.ts';

export function OrganisationSwitcher() {
  const { isMobile } = useSidebar();
  const [isAddOrganisationDialogOpen, setIsAddOrganisationDialogOpen] = React.useState(false);

  const { organisations, isLoading, createOrganisation } = useOrganization();

  const [activeOrganisation, setActiveOrganisation] = React.useState<OrganizationResponse | null>(
    null,
  );

  //todo: fix
  const orgList = organisations?.organizations ?? [];

  React.useEffect(() => {
    if (!activeOrganisation && orgList.length > 0) {
      setActiveOrganisation(orgList[0]);
    }
  }, [orgList, activeOrganisation]);

  const handleCreate = (name: string) => {
    createOrganisation(name, {
      onSuccess: () => {
        setIsAddOrganisationDialogOpen(false);
      },
    });
  };

  if (isLoading) {
    return <div className='p-4 animate-pulse text-sm text-muted-foreground'>Chargement...</div>;
  }

  const current = activeOrganisation || orgList[0];

  return (
    <>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton
                size='lg'
                className='data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground'
              >
                <div className='bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg'>
                  <Building2 className='size-4' />
                </div>

                <div className='grid flex-1 text-left text-sm leading-tight'>
                  <span className='truncate font-medium'>{current?.name ?? 'Sélectionner'}</span>
                  <span className='truncate text-xs text-muted-foreground'>
                    {current?.description || 'Organisation'}
                  </span>
                </div>
                <ChevronsUpDown className='ml-auto' />
              </SidebarMenuButton>
            </DropdownMenuTrigger>

            <DropdownMenuContent
              className='w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg'
              align='start'
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className='text-muted-foreground text-xs'>
                Organisations
              </DropdownMenuLabel>

              {orgList.map((organisation, index) => (
                <DropdownMenuItem
                  key={organisation.organizationId}
                  onClick={() => setActiveOrganisation(organisation)}
                  className='gap-2 p-2 cursor-pointer'
                >
                  <div className='flex size-6 items-center justify-center rounded-md border'>
                    <Building2 className='size-3.5 shrink-0' />
                  </div>
                  <span className='flex-1 truncate'>{organisation.name}</span>
                  <DropdownMenuShortcut>⌘{index + 1}</DropdownMenuShortcut>
                </DropdownMenuItem>
              ))}

              <DropdownMenuSeparator />

              <DropdownMenuItem
                className='gap-2 p-2 cursor-pointer'
                onClick={() => setIsAddOrganisationDialogOpen(true)}
              >
                <div className='flex size-6 items-center justify-center rounded-md border bg-transparent'>
                  <Plus className='size-4' />
                </div>
                <div className='text-muted-foreground font-medium'>Create a organization</div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>

      <AddOrganizationDialog
        open={isAddOrganisationDialogOpen}
        onOpenChange={setIsAddOrganisationDialogOpen}
        onAddOrganization={handleCreate}
      />
    </>
  );
}
