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
                  bg-white dark:bg-slate-900
                  border border-slate-200 dark:border-slate-700
                  rounded-lg
                  hover:scale-105
                  hover:bg-slate-50 dark:hover:bg-slate-800
                  hover:border-slate-300 dark:hover:border-slate-600
                  data-[state=open]:bg-slate-50 dark:data-[state=open]:bg-slate-800
                  data-[state=open]:scale-105
                  data-[state=open]:border-primary dark:data-[state=open]:border-primary-border
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
                rounded-xl border-slate-200 dark:border-slate-700 shadow-lg
              '
              align='start'
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className='text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider px-3 py-2'>
                {label}
              </DropdownMenuLabel>

              <List Wrapper={DropdownMenuItem} />

              <DropdownMenuSeparator className='bg-slate-100 dark:bg-slate-800' />

              <DropdownMenuItem
                onSelect={() => setOpen(true)}
                className='gap-3 p-2 mx-1 mb-1 rounded-lg cursor-pointer hover:bg-primary-hover dark:hover:bg-primary-hover group'
              >
                <div className='flex size-8 items-center justify-center rounded-md border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 group-hover:border-primary dark:group-hover:border-primary-border transition-colors'>
                  <Plus className='size-4 text-slate-500 dark:text-slate-400 group-hover:text-primary dark:group-hover:text-primary-light transition-colors' />
                </div>
                <div className='font-medium text-slate-700 dark:text-slate-300 group-hover:text-primary dark:group-hover:text-primary-light'>
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
