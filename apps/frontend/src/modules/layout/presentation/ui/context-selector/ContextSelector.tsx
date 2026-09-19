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
  /**
   * The rows, already rendered. They used to be a component taking a row
   * wrapper so this file could inject `DropdownMenuItem`; the list now renders
   * its own, because a Svelte list cannot be handed a React component and the
   * indirection bought nothing else.
   */
  list: ReactNode;
  addModal: ComponentType<{ open: boolean; setOpen: (open: boolean) => void }>;
  /** When false, the "Create new …" entry is hidden. Defaults to allowed. */
  canAdd?: boolean;
};

export const ContextSelector = ({
  label,
  display,
  list,
  addModal: AddModal,
  canAdd = true,
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
                  rounded-lg px-2
                  hover:bg-accent
                  data-[state=open]:bg-accent
                  transition-colors duration-200
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

              {list}

              {canAdd && (
                <>
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
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <AddModal open={open} setOpen={setOpen} />
    </>
  );
};
