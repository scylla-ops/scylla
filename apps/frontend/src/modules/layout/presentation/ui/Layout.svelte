<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { NavEntry } from '@platform/routing';
  import { SidebarInset, SidebarProvider } from '@shadcn';
  import { ScyllaLoadingScreen } from '@shared/presentation/ui';
  import { createShellState } from '../shell.state.svelte.ts';
  import AppSidebar from './AppSidebar.svelte';
  import FirstOrganization from './FirstOrganization.svelte';
  import TopBar from './TopBar.svelte';
  import WhatsNewDialog from './WhatsNewDialog.svelte';

  interface Props {
    navEntries: readonly NavEntry[];
    children: Snippet;
  }

  let { navEntries, children }: Props = $props();

  const shell = createShellState();
</script>

{#if shell.isLoading}
  <ScyllaLoadingScreen />
{:else if !shell.hasOrganizations}
  <FirstOrganization isPending={shell.isCreating} onSubmit={shell.createFirstOrganization} />
{:else}
  <SidebarProvider class="h-svh w-full">
    <AppSidebar {navEntries} />
    <SidebarInset
      class="flex min-w-0 flex-1 flex-col border border-sidebar-border bg-background p-2"
    >
      <TopBar />
      <div class="min-h-0 flex-1 overflow-y-auto p-2">
        {@render children()}
      </div>
    </SidebarInset>
    <WhatsNewDialog />
  </SidebarProvider>
{/if}
