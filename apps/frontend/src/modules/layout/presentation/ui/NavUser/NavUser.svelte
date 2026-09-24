<script lang="ts">
  import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
  import LogOutIcon from '@lucide/svelte/icons/log-out';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import { scyllaNavigate } from '@platform/context';
  import { createQuery } from '@platform/query';
  import {
    Avatar,
    AvatarFallback,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
    getSidebar,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from '@shadcn';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userQueries } from '@/modules/features/user';
  import { signOut } from '../../sign-out.ts';
  import { layoutMessages } from '../layout.messages.ts';

  const sidebar = getSidebar();
  const userId = localStorage.getItem('userId') || undefined;
  const userQuery = createQuery(() => userQueries.byId(userId));
  const user = $derived(userQuery.data);
</script>

{#if userQuery.isLoading}
  <div>{t(layoutMessages.loading)}</div>
{:else}
  <SidebarMenu>
    <SidebarMenuItem
      class="rounded-lg border border-border bg-background shadow-sm transition-all duration-200 hover:scale-105 hover:border-primary/40 hover:bg-accent hover:shadow-md focus:outline-none focus:ring-0 focus-visible:ring-0 data-[state=open]:scale-105 data-[state=open]:border-primary data-[state=open]:bg-accent"
    >
      <DropdownMenu>
        <DropdownMenuTrigger>
          {#snippet child({ props })}
            <SidebarMenuButton
              {...props}
              size="lg"
              class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <Avatar class="h-8 w-8 rounded-lg">
                <AvatarFallback class="rounded-lg">
                  {user?.username.at(0)?.toUpperCase()}
                </AvatarFallback>
              </Avatar>
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-medium">{user?.username}</span>
              </div>
              <ChevronsUpDownIcon class="ml-auto size-4" />
            </SidebarMenuButton>
          {/snippet}
        </DropdownMenuTrigger>
        <DropdownMenuContent
          class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg border-border bg-background shadow-lg"
          side={sidebar.isMobile ? 'bottom' : 'right'}
          align="end"
          sideOffset={4}
        >
          <DropdownMenuItem
            class="p-0 font-normal text-foreground hover:bg-accent"
            onSelect={() => {
              if (user) scyllaNavigate.goToUserSettings(user.userId);
            }}
          >
            <div class="flex w-full items-center gap-2 px-1 py-1.5 text-left text-sm">
              <SettingsIcon class="size-4" />
              <span>{t(layoutMessages.settings)}</span>
            </div>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem class="text-foreground hover:bg-accent" onSelect={signOut}>
            <LogOutIcon />
            {t(layoutMessages.logOut)}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuItem>
  </SidebarMenu>
{/if}
