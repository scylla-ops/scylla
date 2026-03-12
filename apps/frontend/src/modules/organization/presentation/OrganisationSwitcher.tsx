'use client';

import * as React from 'react';
import { ChevronsUpDown, Plus } from 'lucide-react';

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
import { AddOrganizationDialog } from '@/modules/organization/presentation/AddOrganizationDialog.tsx';

export function OrganisationSwitcher({
  organisations,
}: {
  organisations: {
    name: string;
    logo: React.ElementType;
    description: string;
  }[];
}) {
  const { isMobile } = useSidebar();
  const [activeOrganisation, setActiveOrganisation] = React.useState(organisations[0]);
  const [isAddOrganisationDialogOpen, setIsAddOrganisationDialogOpen] = React.useState(false);

  if (!activeOrganisation) {
    return null;
  }

  const handleAddOrganisation = async (organization: { name: string; description: string }) => {
    // TODO: Implement API call to add organization and refresh the list
    organisations.push({ ...organization, logo: Plus });
    console.log('Adding new organization:', organization);
    // After successfully adding organization, you can update the organisations array
    // Example: setOrganisations([...organisations, newOrganization]);
  };

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
                  <activeOrganisation.logo className='size-4' />
                </div>
                <div className='grid flex-1 text-left text-sm leading-tight'>
                  <span className='truncate font-medium'>{activeOrganisation.name}</span>
                  <span className='truncate text-xs'>{activeOrganisation.description}</span>
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
              {organisations.map((organisation, index) => (
                <DropdownMenuItem
                  key={organisation.name}
                  onClick={() => setActiveOrganisation(organisation)}
                  className='gap-2 p-2'
                >
                  <div className='flex size-6 items-center justify-center rounded-md border'>
                    <organisation.logo className='size-3.5 shrink-0' />
                  </div>
                  {organisation.name}
                  <DropdownMenuShortcut>⌘{index + 1}</DropdownMenuShortcut>
                </DropdownMenuItem>
              ))}
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className='gap-2 p-2'
                onClick={() => setIsAddOrganisationDialogOpen(true)}
              >
                <div className='flex size-6 items-center justify-center rounded-md border bg-transparent'>
                  <Plus className='size-4' />
                </div>
                <div className='text-muted-foreground font-medium'>Add organisation</div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <AddOrganizationDialog
        open={isAddOrganisationDialogOpen}
        onOpenChange={setIsAddOrganisationDialogOpen}
        onAddOrganization={handleAddOrganisation}
      />
    </>
  );
}
