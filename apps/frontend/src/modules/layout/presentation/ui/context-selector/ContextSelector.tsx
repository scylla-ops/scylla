import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/modules/shared/presentation/ui/shadcn/dropdown-menu.tsx';
import { Plus } from 'lucide-react';
import { type ComponentType, type ReactNode, useState } from 'react';

type ContextSelectorProps = {
  label: string;
  display: ReactNode;
  list: ComponentType<{
    Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void; className?: string }>;
  }>;
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
              <SidebarMenuButton
                size='lg'
                className='
                  bg-background
                  border border-border
                  rounded-lg
                  hover:scale-105
                  hover:bg-accent
                  hover:border-primary/40
                  data-[state=open]:bg-accent
                  data-[state=open]:scale-105
                  data-[state=open]:border-primary
                  transition-all duration-200
                  shadow-sm hover:shadow-md
                  focus:ring-0 focus:outline-none focus-visible:ring-0
                '
              >
                {display}
              </SidebarMenuButton>
            </DropdownMenuTrigger>

            <DropdownMenuContent
              className='
                w-[--radix-dropdown-menu-trigger-width] min-w-56
                rounded-xl border-border shadow-lg
              '
              align='start'
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className='text-xs font-semibold text-muted-foreground uppercase tracking-wider px-3 py-2'>
                {label}
              </DropdownMenuLabel>

              <List Wrapper={DropdownMenuItem} />

              <DropdownMenuSeparator className='bg-border' />

              <DropdownMenuItem
                onSelect={() => setOpen(true)}
                className='gap-3 p-2 mx-1 mb-1 rounded-lg cursor-pointer hover:bg-accent group'
              >
                <div className='flex size-8 items-center justify-center rounded-md border border-border bg-background group-hover:border-primary transition-colors'>
                  <Plus className='size-4 text-muted-foreground group-hover:text-primary transition-colors' />
                </div>
                <div className='font-medium text-foreground group-hover:text-primary'>
                  Create new {label.toLowerCase()}
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <AddModal open={open} setOpen={setOpen} />
    </>
  );
};
