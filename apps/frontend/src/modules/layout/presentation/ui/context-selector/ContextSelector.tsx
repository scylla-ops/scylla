import { SidebarMenu, SidebarMenuButton, SidebarMenuItem, useSidebar } from '@shadcn/sidebar.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@shadcn/dropdown-menu.tsx';
import { Plus } from 'lucide-react';
import { type ComponentType, type ReactNode, useState } from 'react';

type ContextSelectorProps = {
  label: string;
  display: ReactNode;
  list: ComponentType<{ Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }> }>;
  addModal: ComponentType<{ open: boolean; setOpen: (open: boolean) => void }>;
};

export const ContextSelector = ({
  label,
  display,
  list: List,
  addModal: AddModal,
}: ContextSelectorProps) => {
  const { isMobile } = useSidebar();
  const [open, setOpen] = useState(false);

  return (
    <>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton size='lg' className='data-[state=open]:bg-sidebar-accent'>
                {display}
              </SidebarMenuButton>
            </DropdownMenuTrigger>

            <DropdownMenuContent
              className='w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg'
              align='start'
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className='text-muted-foreground text-xs'>
                {label}
              </DropdownMenuLabel>

              <List Wrapper={DropdownMenuItem} />

              <DropdownMenuSeparator />

              <DropdownMenuItem onSelect={() => setOpen(true)} className='gap-2 p-2 cursor-pointer'>
                <div className='flex size-6 items-center justify-center rounded-md border bg-transparent'>
                  <Plus className='size-4' />
                </div>
                <div className='text-muted-foreground font-medium'>Create</div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <AddModal open={open} setOpen={setOpen} />
    </>
  );
};
