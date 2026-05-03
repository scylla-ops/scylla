import * as React from 'react';
import { Button } from '@shadcn';
import { cn } from '@shared/presentation/utils';
import { ArrowLeftToLine } from 'lucide-react';
import { useSidebar } from '@shadcn/sidebar.tsx';

//Custom from shadcn for scylla
export function ScyllaSidebarTrigger({
  className,
  onClick,
  ...props
}: React.ComponentProps<typeof Button>) {
  const { toggleSidebar, open } = useSidebar();

  return (
    <Button
      data-sidebar='trigger'
      data-slot='sidebar-trigger'
      variant='ghost'
      size='icon'
      className={cn('size-7', className)}
      onClick={event => {
        onClick?.(event);
        toggleSidebar();
      }}
      {...props}
    >
      <ArrowLeftToLine
        className={cn('size-4 transition-transform duration-300', !open && 'rotate-180')}
      />
      <span className='sr-only'>Toggle Sidebar</span>
    </Button>
  );
}

export default ScyllaSidebarTrigger;
