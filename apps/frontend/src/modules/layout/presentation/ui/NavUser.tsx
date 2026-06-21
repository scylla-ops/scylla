import { Trans } from '@lingui/react/macro';
import { ChevronsUpDown, LogOut, SettingsIcon } from 'lucide-react';

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from '@/modules/shared/presentation/ui/shadcn/avatar.tsx';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/modules/shared/presentation/ui/shadcn/dropdown-menu.tsx';
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useUser } from '@/modules/features/user/presentation/hooks/use-user.ts';

export function NavUser() {
  const { isMobile } = useSidebar();
  const goToUserSettings = useScyllaNavigate().goToUserSettings;
  const resetContext = useContextStore(state => state.reset);
  //todo: use context store
  const userId = localStorage.getItem('userId');

  //fixme: dependency to user module here (if we are in layout)
  //todo: handle error properly here
  const { user, isLoading } = useUser(userId || undefined);

  //todo: better loading (skeleton if loading too slow ?)
  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <SidebarMenu>
      <SidebarMenuItem
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
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size='lg'
              className='data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground'
            >
              <Avatar className='h-8 w-8 rounded-lg'>
                <AvatarImage />
                <AvatarFallback className='rounded-lg'>
                  {user?.username.at(0)?.toUpperCase()}
                </AvatarFallback>
              </Avatar>

              <div className='grid flex-1 text-left text-sm leading-tight'>
                <span className='truncate font-medium'>{user?.username}</span>
              </div>
              <ChevronsUpDown className='ml-auto size-4' />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className='w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg'
            side={isMobile ? 'bottom' : 'right'}
            align='end'
            sideOffset={4}
          >
            <DropdownMenuItem
              onSelect={() => goToUserSettings(user?.userId)}
              className='p-0 font-normal'
            >
              <div className='w-full flex items-center gap-2 px-1 py-1.5 text-left text-sm'>
                <SettingsIcon className='size-4' />
                <span>Settings</span>
              </div>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onSelect={() => {
                localStorage.removeItem('token');
                resetContext();
                window.location.href = '/';
              }}
            >
              <LogOut />
              <Trans>Log out</Trans>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
